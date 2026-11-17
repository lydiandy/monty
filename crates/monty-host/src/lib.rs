//! The host's own values, owned by one session
//!
//! A value crossing between Python and the host is either an immediate
//! (`None`, a bool, a number, a string, bytes) or a handle into this arena:
//! a dict, a list, a module being built, or a host object (an element, a
//! signal, a component, a class instance). The arena is ours, not the
//! interpreter's: `MontyObject` is converted at the boundary and nothing here
//! holds a Python reference.
//!
//! References are counted the way the boundary uses them: every `alloc`
//! returns one owned reference, `inc_ref` copies it, `dec_ref` drops it, and
//! the entry is freed at zero. A value the host keeps (a callback, a prop's
//! list) keeps its entry alive.

use std::{cell::RefCell, fmt};

use monty_types::MontyUuid;

/// The class type a host handle crosses as
pub(crate) const HANDLE_CLASS: &str = "nsh.handle";
/// The uuid of the handle class type itself
pub(crate) const HANDLE_CLASS_UUID: MontyUuid = MontyUuid::from_u128(u128::MAX - 1);

/// A host object: its kind and the index of its Rust value
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostObject {
    /// The host object's kind
    pub kind: u16,
    /// The index of its Rust value
    pub data: u64,
}

/// A handle into the session's [`Arena`]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct HeapId {
    index: u32,
    generation: u32,
}

impl HeapId {
    /// The slot index, for diagnostics
    pub(crate) const fn index(self) -> u32 {
        self.index
    }

    /// The boundary uuid this handle crosses as
    ///
    /// The top 32 bits tag an NSH handle, so a uuid from a class or a
    /// module never decodes as one.
    pub(crate) const fn uuid(self) -> monty_types::MontyUuid {
        let value =
            (0x4E53_4800u128 << 96) | ((self.generation as u128) << 32) | self.index as u128;
        monty_types::MontyUuid::from_u128(value)
    }

    /// The handle a boundary uuid names, if it is one
    pub(crate) fn from_uuid(uuid: &monty_types::MontyUuid) -> Option<Self> {
        let value = u128::from_be_bytes(*uuid.as_bytes());
        if value >> 96 != 0x4E53_4800 {
            return None;
        }
        Some(Self {
            index: value as u32,
            generation: (value >> 32) as u32,
        })
    }
}

impl fmt::Debug for HeapId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HeapId({}:{})", self.index, self.generation)
    }
}

/// A sandbox callable the host holds
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Callback(pub(crate) monty_types::MontyUuid);

/// A value at the boundary
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Bytes(Vec<u8>),
    /// A dict, list, module or host object in the arena
    Heap(HeapId),
    /// A sandbox function the host holds
    Func(Callback),
}

impl Value {
    pub(crate) fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(value) => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(value) => Some(*value),
            Self::Int(value) => Some(*value as f64),
            _ => None,
        }
    }

    pub(crate) fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn as_handle(&self) -> Option<HeapId> {
        match self {
            Self::Heap(id) => Some(*id),
            _ => None,
        }
    }

    pub(crate) fn as_func(&self) -> Option<Callback> {
        match self {
            Self::Func(callback) => Some(*callback),
            _ => None,
        }
    }
}

/// One arena entry
struct Entry {
    stored: Option<Stored>,
    refs: u32,
    generation: u32,
}

enum Stored {
    Dict(Vec<(String, Value)>),
    List(Vec<Value>),
    Object(HostObject),
    Module {
        name: String,
        package: bool,
        attrs: Vec<(String, Value)>,
    },
    Bytes(Vec<u8>),
}

/// The values one session holds
#[derive(Default)]
pub(crate) struct Arena {
    entries: RefCell<Vec<Entry>>,
    free: RefCell<Vec<u32>>,
}

impl Arena {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    fn push(&self, stored: Stored) -> HeapId {
        let mut entries = self.entries.borrow_mut();
        if let Some(index) = self.free.borrow_mut().pop() {
            let entry = &mut entries[index as usize];
            debug_assert!(entry.stored.is_none());
            entry.stored = Some(stored);
            entry.refs = 1;
            return HeapId {
                index,
                generation: entry.generation,
            };
        }
        let index = entries.len() as u32;
        entries.push(Entry {
            stored: Some(stored),
            refs: 1,
            generation: 0,
        });
        HeapId {
            index,
            generation: 0,
        }
    }

    pub(crate) fn alloc_dict(&self, entries: Vec<(String, Value)>) -> HeapId {
        self.push(Stored::Dict(entries))
    }

    pub(crate) fn alloc_list(&self, items: Vec<Value>) -> HeapId {
        self.push(Stored::List(items))
    }

    pub(crate) fn alloc_object(&self, object: HostObject) -> HeapId {
        self.push(Stored::Object(object))
    }

    pub(crate) fn alloc_module(&self, name: &str, package: bool) -> HeapId {
        self.push(Stored::Module {
            name: name.to_owned(),
            package,
            attrs: Vec::new(),
        })
    }

    pub(crate) fn module_set_attr(
        &self,
        id: HeapId,
        name: &str,
        value: Value,
    ) -> Result<(), String> {
        let mut entries = self.entries.borrow_mut();
        let entry = entries
            .get_mut(id.index as usize)
            .filter(|entry| entry.generation == id.generation)
            .ok_or_else(|| "a host value was used after it was released".to_owned())?;
        match entry.stored.as_mut() {
            Some(Stored::Module { attrs, .. }) => {
                attrs.push((name.to_owned(), value));
                Ok(())
            }
            Some(_) => Err("only a module takes attributes".to_owned()),
            None => Err("a host value was used after it was released".to_owned()),
        }
    }

    pub(crate) fn dict_get(&self, id: HeapId, key: &str) -> Option<Value> {
        let entries = self.entries.borrow();
        match entries.get(id.index as usize) {
            Some(Entry {
                stored: Some(Stored::Dict(entries)),
                generation,
                ..
            }) if *generation == id.generation => entries
                .iter()
                .find_map(|(name, value)| (name == key).then_some(value.clone())),
            _ => None,
        }
    }

    pub(crate) fn dict_key_at(&self, id: HeapId, index: usize) -> Option<Value> {
        let entries = self.entries.borrow();
        match entries.get(id.index as usize) {
            Some(Entry {
                stored: Some(Stored::Dict(entries)),
                generation,
                ..
            }) if *generation == id.generation => {
                entries.get(index).map(|(key, _)| Value::Str(key.clone()))
            }
            _ => None,
        }
    }

    pub(crate) fn dict_set(&self, id: HeapId, key: &str, value: Value) -> Result<(), String> {
        let mut entries = self.entries.borrow_mut();
        let entry = entries
            .get_mut(id.index as usize)
            .filter(|entry| entry.generation == id.generation)
            .ok_or_else(|| "a host value was used after it was released".to_owned())?;
        match entry.stored.as_mut() {
            Some(Stored::Dict(entries)) => {
                if let Some(pair) = entries.iter_mut().find(|(name, _)| name == key) {
                    pair.1 = value;
                } else {
                    entries.push((key.to_owned(), value));
                }
                Ok(())
            }
            Some(_) => Err("only a dict takes string keys".to_owned()),
            None => Err("a host value was used after it was released".to_owned()),
        }
    }

    pub(crate) fn dict_len(&self, id: HeapId) -> Option<usize> {
        let entries = self.entries.borrow();
        match entries.get(id.index as usize) {
            Some(Entry {
                stored: Some(Stored::Dict(entries)),
                generation,
                ..
            }) if *generation == id.generation => Some(entries.len()),
            _ => None,
        }
    }

    pub(crate) fn list_get(&self, id: HeapId, index: usize) -> Option<Value> {
        let entries = self.entries.borrow();
        match entries.get(id.index as usize) {
            Some(Entry {
                stored: Some(Stored::List(items)),
                generation,
                ..
            }) if *generation == id.generation => items.get(index).cloned(),
            _ => None,
        }
    }

    pub(crate) fn list_len(&self, id: HeapId) -> Option<usize> {
        let entries = self.entries.borrow();
        match entries.get(id.index as usize) {
            Some(Entry {
                stored: Some(Stored::List(items)),
                generation,
                ..
            }) if *generation == id.generation => Some(items.len()),
            _ => None,
        }
    }

    pub(crate) fn alloc_bytes(&self, bytes: Vec<u8>) -> HeapId {
        self.push(Stored::Bytes(bytes))
    }

    pub(crate) fn bytes_get(&self, id: HeapId) -> Option<Vec<u8>> {
        let entries = self.entries.borrow();
        match entries.get(id.index as usize) {
            Some(Entry {
                stored: Some(Stored::Bytes(bytes)),
                generation,
                ..
            }) if *generation == id.generation => Some(bytes.clone()),
            _ => None,
        }
    }

    pub(crate) fn object(&self, id: HeapId) -> Option<HostObject> {
        let entries = self.entries.borrow();
        match entries.get(id.index as usize) {
            Some(Entry {
                stored: Some(Stored::Object(object)),
                generation,
                ..
            }) if *generation == id.generation => Some(*object),
            _ => None,
        }
    }

    pub(crate) fn module(&self, id: HeapId) -> Option<(String, bool, Vec<(String, Value)>)> {
        let entries = self.entries.borrow();
        match entries.get(id.index as usize) {
            Some(Entry {
                stored:
                    Some(Stored::Module {
                        name,
                        package,
                        attrs,
                    }),
                generation,
                ..
            }) if *generation == id.generation => Some((name.clone(), *package, attrs.clone())),
            _ => None,
        }
    }

    pub(crate) fn is_list(&self, id: HeapId) -> bool {
        let entries = self.entries.borrow();
        matches!(
            entries.get(id.index as usize),
            Some(Entry { stored: Some(Stored::List(_)), generation, .. }) if *generation == id.generation
        )
    }

    pub(crate) fn is_dict(&self, id: HeapId) -> bool {
        let entries = self.entries.borrow();
        matches!(
            entries.get(id.index as usize),
            Some(Entry { stored: Some(Stored::Dict(_)), generation, .. }) if *generation == id.generation
        )
    }

    pub(crate) fn inc_ref(&self, id: HeapId) {
        let mut entries = self.entries.borrow_mut();
        if let Some(entry) = entries
            .get_mut(id.index as usize)
            .filter(|entry| entry.generation == id.generation)
        {
            entry.refs = entry.refs.saturating_add(1);
        }
    }

    pub(crate) fn dec_ref(&self, id: HeapId) {
        let mut entries = self.entries.borrow_mut();
        let Some(entry) = entries
            .get_mut(id.index as usize)
            .filter(|entry| entry.generation == id.generation)
        else {
            return;
        };
        if entry.refs > 1 {
            entry.refs -= 1;
            return;
        }
        let Some(stored) = entry.stored.take() else {
            return;
        };
        entry.refs = 0;
        entry.generation = entry.generation.wrapping_add(1);
        let index = id.index;
        drop(entries);
        let children: Vec<Value> = match &stored {
            Stored::List(items) => items.clone(),
            Stored::Dict(entries) => entries.iter().map(|(_, value)| value.clone()).collect(),
            Stored::Module { attrs, .. } => attrs.iter().map(|(_, value)| value.clone()).collect(),
            Stored::Object(_) | Stored::Bytes(_) => Vec::new(),
        };
        for child in children {
            if let Value::Heap(child) = child {
                self.dec_ref(child);
            }
        }
        self.free.borrow_mut().push(index);
    }
}

#[cfg(test)]
mod tests {
    use super::{Arena, HostObject, Value};

    #[test]
    fn dicts_and_lists_are_counted_and_freed() {
        let arena = Arena::new();
        let list = arena.alloc_list(vec![Value::Int(1), Value::Int(2)]);
        let dict = arena.alloc_dict(vec![("items".into(), Value::Heap(list))]);
        arena.inc_ref(list);
        assert_eq!(arena.list_len(list), Some(2));
        assert_eq!(arena.list_get(list, 1), Some(Value::Int(2)));
        arena.dec_ref(dict);
        // The list still has the copy the test took.
        assert_eq!(arena.list_len(list), Some(2));
        arena.dec_ref(list);
        arena.dec_ref(list);
        assert_eq!(arena.list_len(list), None);
    }

    #[test]
    fn objects_and_modules_cross() {
        let arena = Arena::new();
        let object = arena.alloc_object(HostObject { kind: 3, data: 9 });
        assert_eq!(arena.object(object), Some(HostObject { kind: 3, data: 9 }));
        let module = arena.alloc_module("nsh.base", true);
        arena
            .module_set_attr(module, "text", Value::Str("text".into()))
            .unwrap();
        let (name, package, attrs) = arena.module(module).unwrap();
        assert_eq!(name, "nsh.base");
        assert!(package);
        assert_eq!(attrs.len(), 1);
    }

    #[test]
    fn a_released_handle_does_not_alias_a_new_value() {
        let arena = Arena::new();
        let first = arena.alloc_list(vec![]);
        arena.dec_ref(first);
        let second = arena.alloc_list(vec![]);
        assert_eq!(first.index(), second.index());
        assert_eq!(arena.list_len(first), None);
        assert_eq!(arena.list_len(second), Some(0));
    }
}

/// The host's value operations, as the Python API's functions use them
///
/// Implemented by the session, over the arena; the old interpreter's
/// `HostCtx` trait is gone, and nothing here reaches the interpreter.
pub trait HostCtx {
    fn alloc(&self, object: HostObject) -> HeapId;
    fn alloc_dict(&self, _entries: Vec<(String, Value)>) -> Result<HeapId, String> {
        Err("this host holds no dicts".to_owned())
    }
    fn alloc_list(&self, _items: Vec<Value>) -> Result<HeapId, String> {
        Err("this host holds no lists".to_owned())
    }
    fn alloc_module(&self, _name: &str) -> Result<HeapId, String> {
        Err("this host holds no modules".to_owned())
    }
    fn alloc_package(&self, _name: &str) -> Result<HeapId, String> {
        Err("this host holds no packages".to_owned())
    }
    fn module_set_attr(&self, _module: HeapId, _name: &str, _value: Value) -> Result<(), String> {
        Err("this host holds no modules".to_owned())
    }
    fn dict_get(&self, _id: HeapId, _key: &str) -> Option<Value> {
        None
    }
    fn dict_key_at(&self, _id: HeapId, _index: usize) -> Option<Value> {
        None
    }
    fn dict_len(&self, _id: HeapId) -> Option<usize> {
        None
    }
    fn list_get(&self, _id: HeapId, _index: usize) -> Option<Value> {
        None
    }
    fn list_len(&self, _id: HeapId) -> Option<usize> {
        None
    }
    fn is_list(&self, _id: HeapId) -> bool {
        false
    }
    fn is_dict(&self, _id: HeapId) -> bool {
        false
    }
    fn host_object(&self, _id: HeapId) -> Option<HostObject> {
        None
    }
    fn inc_ref(&self, _id: HeapId) {}
    fn dec_ref(&self, _id: HeapId) {}
    fn bytes_get(&self, _id: HeapId) -> Option<Vec<u8>> {
        None
    }
    fn alloc_bytes(&self, _bytes: Vec<u8>) -> HeapId {
        unreachable!("this host holds no bytes")
    }
    /// Calls a sandbox callback the host holds
    fn call_func(&self, _callback: Callback, _args: Vec<Value>) -> Result<Value, String> {
        Err("this host has no sandbox".to_owned())
    }
    /// Starts a host await and returns its call id and the awaitable Python holds
    fn host_await(&self) -> Result<(u32, Value), String> {
        Err("this host has no sandbox".to_owned())
    }
}

impl HostCtx for Arena {
    fn alloc(&self, object: HostObject) -> HeapId {
        self.alloc_object(object)
    }

    fn alloc_dict(&self, entries: Vec<(String, Value)>) -> Result<HeapId, String> {
        Ok(self.alloc_dict(entries))
    }

    fn alloc_list(&self, items: Vec<Value>) -> Result<HeapId, String> {
        Ok(self.alloc_list(items))
    }

    fn alloc_module(&self, name: &str) -> Result<HeapId, String> {
        Ok(self.alloc_module(name, false))
    }

    fn alloc_package(&self, name: &str) -> Result<HeapId, String> {
        Ok(self.alloc_module(name, true))
    }

    fn module_set_attr(&self, module: HeapId, name: &str, value: Value) -> Result<(), String> {
        self.module_set_attr(module, name, value)
    }

    fn dict_get(&self, id: HeapId, key: &str) -> Option<Value> {
        self.dict_get(id, key)
    }

    fn dict_key_at(&self, id: HeapId, index: usize) -> Option<Value> {
        self.dict_key_at(id, index)
    }

    fn dict_len(&self, id: HeapId) -> Option<usize> {
        self.dict_len(id)
    }

    fn list_get(&self, id: HeapId, index: usize) -> Option<Value> {
        self.list_get(id, index)
    }

    fn list_len(&self, id: HeapId) -> Option<usize> {
        self.list_len(id)
    }

    fn is_list(&self, id: HeapId) -> bool {
        self.is_list(id)
    }

    fn is_dict(&self, id: HeapId) -> bool {
        self.is_dict(id)
    }

    fn host_object(&self, id: HeapId) -> Option<HostObject> {
        self.object(id)
    }

    fn inc_ref(&self, id: HeapId) {
        self.inc_ref(id);
    }

    fn dec_ref(&self, id: HeapId) {
        self.dec_ref(id);
    }

    fn bytes_get(&self, id: HeapId) -> Option<Vec<u8>> {
        self.bytes_get(id)
    }

    fn alloc_bytes(&self, bytes: Vec<u8>) -> HeapId {
        self.alloc_bytes(bytes)
    }

    fn call_func(&self, _: Callback, _: Vec<Value>) -> Result<Value, String> {
        Err("this host has no sandbox to call".to_owned())
    }

    fn host_await(&self) -> Result<(u32, Value), String> {
        Err("this host has no sandbox to await in".to_owned())
    }
}

/// What a host answers when Python calls one of its objects
///
/// The app's host is [`crate::host::MshVtable`]; a test file's VM plugs its
/// own in, delegating the runtime's kinds to a [`crate::ModulesHost`].
pub trait Dispatch: 'static {
    fn call_attr(
        &self,
        ctx: &mut dyn HostCtx,
        id: HeapId,
        object: HostObject,
        attr: &str,
        args: Vec<Value>,
        kwargs: Vec<(String, Value)>,
    ) -> Result<Value, String>;

    fn getattr(
        &self,
        ctx: &mut dyn HostCtx,
        id: HeapId,
        object: HostObject,
        attr: &str,
    ) -> Result<Option<Value>, String>;

    fn call(
        &self,
        ctx: &mut dyn HostCtx,
        id: HeapId,
        object: HostObject,
        args: Vec<Value>,
        kwargs: Vec<(String, Value)>,
    ) -> Result<Value, String>;

    fn construct(
        &self,
        ctx: &mut dyn HostCtx,
        name: &str,
        args: Vec<Value>,
    ) -> Result<Value, String>;

    fn create_module(&self, ctx: &mut dyn HostCtx, name: &str) -> Result<Option<HeapId>, String>;
}

/// The host value, under the name embedders use
pub use Value as HostValue;
