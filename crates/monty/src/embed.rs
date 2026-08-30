//! In-process embedding API for hosts that need `VM` / `Heap` / `HeapId`.
//!
//! The published `MontyObject` / `MontyRepl` surface cannot hold instances
//! (they become `Repr` strings) and has no `call_method`. gpui-monty keeps
//! the interpreter alive and talks to it through this module.
//!
//! Host types stay out of this crate: one [`HostObject`] payload plus a
//! [`HostVtable`] implemented by the embedder.

use std::{
    cell::RefCell,
    mem,
    rc::Rc,
};

use monty_types::{CompileOptions, MontyException, PrintWriter, ResourceTracker};

use crate::{
    args::{ArgValues, KwargsValues},
    bytecode::{CallResult, VM},
    exception_private::{ExcType, RunError, RunResult, SimpleException},
    heap::{Heap, HeapData, HeapId, HeapObjectRead, HeapReader},
    run::Executor,
    types::{Dict, Instance, PyTrait, Type, allocate_string},
    value::Value,
};

pub use crate::host_modules::HostModuleSource;

/// Element handle recorded into a spec arena.
pub const KIND_ELEMENT: u16 = 1;
/// `Button` type object (`Button.new(...)`).
pub const KIND_BUTTON_TYPE: u16 = 2;
/// Per-call `cx` handle.
pub const KIND_CX: u16 = 3;
/// `cx.theme()` result.
pub const KIND_THEME: u16 = 4;
/// `cx.theme().colors`.
pub const KIND_COLORS: u16 = 5;
/// Nested `@view` handle returned by `cx.new(Class, props)`.
pub const KIND_VIEW: u16 = 6;
/// `Checkbox` type object (`Checkbox.new(...)`).
pub const KIND_CHECKBOX_TYPE: u16 = 7;
/// `Switch` type object (`Switch.new(...)`).
pub const KIND_SWITCH_TYPE: u16 = 8;
/// `Link` type object (`Link.new(...)`).
pub const KIND_LINK_TYPE: u16 = 9;
/// `InputState` type object. `.new` is a documented stub until a retained entity exists.
pub const KIND_INPUT_STATE_TYPE: u16 = 10;

/// Host-owned object: a kind tag plus a host-defined payload word.
///
/// `data` is typically a spec-arena node id. Callbacks live in the embedder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HostObject {
    /// Discriminant chosen by the embedder (`KIND_ELEMENT`, `KIND_CX`, …).
    pub kind: u16,
    /// Embedder payload (for example a spec-arena node id).
    pub data: u64,
}

/// Value crossing the host ↔ VM boundary.
#[derive(Debug, Clone)]
pub enum HostValue {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    /// An owned heap reference. The receiver owns one `inc_ref`.
    Heap(HeapId),
}

/// Heap operations the host may perform while handling a call.
pub trait HostCtx {
    fn alloc(&mut self, obj: HostObject) -> HeapId;
    fn inc_ref(&mut self, id: HeapId);
    fn dec_ref(&mut self, id: HeapId);
    fn host_object(&self, id: HeapId) -> Option<HostObject>;
    /// True when `id` is a class marked `@view` (`__gpui_view__ is True`).
    fn is_view_class(&self, id: HeapId) -> bool;
}

/// Embedder callbacks for `HostObject` methods and StandardLib constructors.
pub trait HostVtable: 'static {
    fn call_attr(
        &mut self,
        ctx: &mut dyn HostCtx,
        id: HeapId,
        obj: HostObject,
        attr: &str,
        args: Vec<HostValue>,
    ) -> Result<HostValue, String>;

    fn getattr(
        &mut self,
        ctx: &mut dyn HostCtx,
        id: HeapId,
        obj: HostObject,
        attr: &str,
    ) -> Result<Option<HostValue>, String>;

    fn construct(
        &mut self,
        ctx: &mut dyn HostCtx,
        name: &str,
        args: Vec<HostValue>,
    ) -> Result<HostValue, String>;

    /// A call the host wants the VM to run *after* `call_attr` returns.
    ///
    /// Used for `.hover(fn)` / `.active(fn)` / `.focus(fn)`: the host opens a
    /// detached style node, then the user callable must run without the host
    /// `RefCell` still borrowed (the callable's own `.bg(...)` re-enters
    /// `call_attr`). Default: nothing pending.
    fn take_pending_call(&mut self) -> Option<(HeapId, Vec<HostValue>)> {
        None
    }
}

/// Persistent VM session after loading an application module.
pub struct Embed {
    executor: Executor,
    heap: Heap,
    globals: Vec<Value>,
}

impl Embed {
    /// Compile and run `source` as a module, then park so later calls can
    /// `evaluate_function` without resuming the finished module frame.
    pub fn run_source(
        source: String,
        script_name: &str,
        host: Rc<RefCell<dyn HostVtable>>,
    ) -> Result<Self, MontyException> {
        Self::run_source_with_modules(source, script_name, Vec::new(), host)
    }

    /// Compile `source` together with host app-dir modules.
    ///
    /// `extra` entries are top-level modules: name `"ui"` loads as `from ui import …`.
    pub fn run_source_with_modules(
        source: String,
        script_name: &str,
        extra: Vec<HostModuleSource>,
        host: Rc<RefCell<dyn HostVtable>>,
    ) -> Result<Self, MontyException> {
        let executor = Executor::new_with_host_modules(
            source,
            script_name,
            Vec::new(),
            CompileOptions::default(),
            extra,
        )?;
        let mut heap = Heap::new(executor.namespace_size(), ResourceTracker::default());
        heap.set_host(host);
        let globals = executor.empty_globals();
        let mut embed = Self {
            executor,
            heap,
            globals,
        };
        embed.run_module()?;
        Ok(embed)
    }

    /// Call a module-global function by name.
    pub fn call_global(
        &mut self,
        name: &str,
        args: Vec<HostValue>,
    ) -> Result<HostValue, MontyException> {
        let name_id = self
            .executor
            .interns
            .get_string_id_by_name(name)
            .ok_or_else(|| MontyException::runtime_error(format!("no global {name}")))?;
        let slot = self
            .executor
            .globals
            .get(name_id)
            .ok_or_else(|| MontyException::runtime_error(format!("no global {name}")))?
            .index();
        self.with_idle_vm(move |vm| {
            let func = vm.globals[slot].clone_with_heap(vm);
            let arg_values = host_values_to_args(vm, args)?;
            let result = vm
                .evaluate_function("call_global", &func, arg_values)
                .map_err(|e| e.into_python_exception(vm.interns, |_| Some("")))?;
            func.drop_with(vm);
            value_to_host(vm, result).map_err(run_err_to_monty)
        })
    }

    /// Scan entry-module globals for the unique class marked `@view`.
    ///
    /// Classes defined in host modules (`row.py`) and imported into the entry
    /// (`from row import Row`) still carry `__gpui_view__`, but they are nested
    /// views, not the application root.
    pub fn find_view_class(&self) -> Result<HeapId, MontyException> {
        let imported = self.imported_view_classes();
        let mut found = Vec::new();
        for value in &self.globals {
            let Value::Ref(id) = value else {
                continue;
            };
            if imported.iter().any(|imported_id| imported_id == id) {
                continue;
            }
            if class_is_view(&self.heap, &self.executor.interns, *id) {
                found.push(*id);
            }
        }
        match found.as_slice() {
            [id] => Ok(*id),
            [] => Err(MontyException::runtime_error(
                "entry file must contain exactly one @view class",
            )),
            _ => Err(MontyException::runtime_error(
                "entry file must contain exactly one @view class (found more than one)",
            )),
        }
    }

    fn imported_view_classes(&self) -> Vec<HeapId> {
        let mut out = Vec::new();
        for module_id in self.executor.host_modules.loaded_module_ids() {
            let HeapData::Module(module) = self.heap.get(module_id) else {
                continue;
            };
            for (_, value) in module.attrs().iter() {
                let Value::Ref(id) = value else {
                    continue;
                };
                if class_is_view(&self.heap, &self.executor.interns, *id) {
                    out.push(*id);
                }
            }
        }
        out
    }

    /// True when `id` is a class marked `@view`.
    pub fn is_view_class(&self, id: HeapId) -> bool {
        class_is_view(&self.heap, &self.executor.interns, id)
    }

    /// True when `receiver` is an instance whose class defines `name`.
    pub fn has_method(&self, receiver: HeapId, name: &str) -> bool {
        instance_has_method(&self.heap, &self.executor.interns, receiver, name)
    }

    /// Allocate an instance of `class` without running user `__init__`.
    pub fn construct_instance(&mut self, class: HeapId) -> Result<HeapId, MontyException> {
        self.with_idle_vm(|vm| {
            let instance_id = vm
                .heap
                .allocate(HeapData::Instance(Box::new(Instance::new(class, Dict::new()))));
            vm.heap.inc_ref(class);
            Ok(instance_id)
        })
    }

    /// Call a method on a heap object (`init`, `render`, a click closure).
    pub fn call_method(
        &mut self,
        receiver: HeapId,
        name: &str,
        args: Vec<HostValue>,
    ) -> Result<HostValue, MontyException> {
        let name = name.to_owned();
        self.with_idle_vm(move |vm| {
            vm.heap.inc_ref(receiver);
            let mut arg_values = host_values_to_args(vm, args)?;
            arg_values = arg_values.prepend(Value::Ref(receiver));
            let func = lookup_method(vm, receiver, &name)?;
            let result = vm
                .evaluate_function("method", &func, arg_values)
                .map_err(|e| e.into_python_exception(vm.interns, |_| Some("")))?;
            func.drop_with(vm);
            value_to_host(vm, result).map_err(run_err_to_monty)
        })
    }

    /// Call a callable heap value (a click `Closure`) with host args (`event`, `cx`).
    pub fn call_callable(
        &mut self,
        callable: HeapId,
        args: Vec<HostValue>,
    ) -> Result<HostValue, MontyException> {
        self.with_idle_vm(move |vm| {
            vm.heap.inc_ref(callable);
            let func = Value::Ref(callable);
            let arg_values = host_values_to_args(vm, args)?;
            let result = vm
                .evaluate_function("<callback>", &func, arg_values)
                .map_err(|e| e.into_python_exception(vm.interns, |_| Some("")))?;
            func.drop_with(vm);
            value_to_host(vm, result).map_err(run_err_to_monty)
        })
    }

    pub fn alloc_host(&mut self, obj: HostObject) -> HeapId {
        self.heap.allocate(HeapData::HostObject(obj))
    }

    pub fn inc_ref(&self, id: HeapId) {
        self.heap.inc_ref(id);
    }

    pub fn dec_ref(&mut self, id: HeapId) {
        self.heap.dec_ref(id);
    }

    pub fn host_object(&self, id: HeapId) -> Option<HostObject> {
        match self.heap.get(id) {
            HeapData::HostObject(obj) => Some(*obj),
            _ => None,
        }
    }

    /// Allocate a dict `{key: value, ...}` of host values (for `init` props).
    pub fn alloc_dict(&mut self, entries: Vec<(String, HostValue)>) -> Result<HeapId, MontyException> {
        self.with_idle_vm(|vm| {
            let dict_id = vm.heap.allocate(HeapData::Dict(Dict::new()));
            match vm.heap.read(dict_id) {
                crate::heap::HeapReadOutput::Dict(mut dict) => {
                    for (key, value) in entries {
                        let k = allocate_string(key, vm.heap);
                        let v = host_value_to_value(vm, value);
                        dict.set(k, v, vm).map_err(run_err_to_monty)?;
                    }
                }
                _ => unreachable!("just allocated a dict"),
            }
            Ok(dict_id)
        })
    }

    /// Run the compiled module to completion (REPL-style branded executor borrow).
    fn run_module(&mut self) -> Result<(), MontyException> {
        let globals = mem::take(&mut self.globals);
        let mut data = (&self.executor, None::<Vec<Value>>);
        let result = HeapReader::with(&mut self.heap, &mut data, |reader, (executor, slot)| {
            let mut vm = VM::new(
                globals,
                &executor.module_code,
                reader,
                &executor.interns,
                PrintWriter::Disabled,
                executor.assert_repr_max_bytes,
            );
            vm.set_host_modules(&executor.host_modules);
            let result = executor
                .run_to_completion(&mut vm)
                .map(|_| ())
                .map_err(|e| {
                    e.into_python_exception(&executor.interns, |_| Some(executor.code.as_str()))
                });
            *slot = Some(vm.take_globals());
            result
        });
        self.globals = data.1.expect("module vm returns globals");
        result
    }

    fn with_idle_vm<R>(
        &mut self,
        f: impl for<'h> FnOnce(&mut VM<'h>) -> Result<R, MontyException>,
    ) -> Result<R, MontyException> {
        let globals = mem::take(&mut self.globals);
        let mut data = (&self.executor, None::<Vec<Value>>);
        let result = HeapReader::with(&mut self.heap, &mut data, |reader, (executor, slot)| {
            let mut vm = VM::new_idle(
                globals,
                &executor.module_code,
                reader,
                &executor.interns,
                PrintWriter::Disabled,
                executor.assert_repr_max_bytes,
            );
            vm.set_host_modules(&executor.host_modules);
            let result = f(&mut vm);
            *slot = Some(vm.take_globals());
            result
        });
        self.globals = data.1.expect("idle vm returns globals");
        result
    }
}

fn class_is_view(heap: &Heap, interns: &crate::intern::Interns, id: HeapId) -> bool {
    let HeapData::Class(class) = heap.get(id) else {
        return false;
    };
    matches!(
        class
            .namespace()
            .get_by_str("__gpui_view__", heap, interns),
        Some(Value::Bool(true))
    )
}

fn instance_has_method(
    heap: &Heap,
    interns: &crate::intern::Interns,
    receiver: HeapId,
    name: &str,
) -> bool {
    let HeapData::Instance(instance) = heap.get(receiver) else {
        return false;
    };
    let HeapData::Class(class) = heap.get(instance.class()) else {
        return false;
    };
    class.namespace().get_by_str(name, heap, interns).is_some()
}

fn lookup_method(vm: &mut VM<'_>, receiver: HeapId, name: &str) -> Result<Value, MontyException> {
    match vm.heap.read(receiver) {
        crate::heap::HeapReadOutput::Instance(instance) => {
            let class_id = instance.get(vm.heap).class();
            let HeapData::Class(class) = vm.heap.get(class_id) else {
                return Err(MontyException::runtime_error(format!(
                    "instance class for {name} is not a Class"
                )));
            };
            class
                .namespace()
                .get_by_str(name, vm.heap, vm.interns)
                .map(|v| v.clone_with_heap(vm.heap))
                .ok_or_else(|| MontyException::runtime_error(format!("no method {name}")))
        }
        crate::heap::HeapReadOutput::Closure(_) | crate::heap::HeapReadOutput::BoundMethod(_) => {
            vm.heap.inc_ref(receiver);
            Ok(Value::Ref(receiver))
        }
        _ => Err(MontyException::runtime_error(format!(
            "cannot call {name} on this object"
        ))),
    }
}

pub(crate) fn dispatch_call_attr(
    vm: &mut VM<'_>,
    id: HeapId,
    obj: HostObject,
    attr: &str,
    args: ArgValues,
) -> RunResult<CallResult> {
    let host = vm
        .heap
        .host()
        .ok_or_else(|| SimpleException::new_msg(ExcType::RuntimeError, "no embedder host attached to the heap"))?;
    let host_args = args_to_host(vm, args)?;
    let mut ctx = VmHostCtx { vm };
    let result = host
        .borrow_mut()
        .call_attr(&mut ctx, id, obj, attr, host_args)
        .map_err(|msg| SimpleException::new_msg(ExcType::RuntimeError, msg))?;
    let pending = host.borrow_mut().take_pending_call();
    let value = host_value_to_value(ctx.vm, result);
    if let Some((callable, args)) = pending {
        let func = Value::Ref(callable);
        let arg_values = host_values_to_args(ctx.vm, args).map_err(run_err_from_monty)?;
        let nested = ctx
            .vm
            .evaluate_function("<host-callback>", &func, arg_values)
            .map_err(|e| e.into_python_exception(ctx.vm.interns, |_| Some("")))?;
        nested.drop_with(ctx.vm);
        func.drop_with(ctx.vm);
    }
    Ok(CallResult::Value(value))
}

pub(crate) fn dispatch_getattr(
    vm: &mut VM<'_>,
    id: HeapId,
    obj: HostObject,
    attr: &str,
) -> RunResult<Option<CallResult>> {
    let host = vm
        .heap
        .host()
        .ok_or_else(|| SimpleException::new_msg(ExcType::RuntimeError, "no embedder host attached to the heap"))?;
    let mut ctx = VmHostCtx { vm };
    let result = host
        .borrow_mut()
        .getattr(&mut ctx, id, obj, attr)
        .map_err(|msg| SimpleException::new_msg(ExcType::RuntimeError, msg))?;
    Ok(result.map(|value| CallResult::Value(host_value_to_value(ctx.vm, value))))
}

pub(crate) fn dispatch_construct(vm: &mut VM<'_>, name: &str, args: ArgValues) -> RunResult<Value> {
    let host = vm
        .heap
        .host()
        .ok_or_else(|| SimpleException::new_msg(ExcType::RuntimeError, "no embedder host attached to the heap"))?;
    let host_args = args_to_host(vm, args)?;
    let mut ctx = VmHostCtx { vm };
    let result = host
        .borrow_mut()
        .construct(&mut ctx, name, host_args)
        .map_err(|msg| SimpleException::new_msg(ExcType::RuntimeError, msg))?;
    Ok(host_value_to_value(ctx.vm, result))
}

struct VmHostCtx<'a, 'h> {
    vm: &'a mut VM<'h>,
}

impl HostCtx for VmHostCtx<'_, '_> {
    fn alloc(&mut self, obj: HostObject) -> HeapId {
        self.vm.heap.allocate(HeapData::HostObject(obj))
    }

    fn inc_ref(&mut self, id: HeapId) {
        self.vm.heap.inc_ref(id);
    }

    fn dec_ref(&mut self, id: HeapId) {
        self.vm.heap.dec_ref(id);
    }

    fn host_object(&self, id: HeapId) -> Option<HostObject> {
        match self.vm.heap.get(id) {
            HeapData::HostObject(obj) => Some(*obj),
            _ => None,
        }
    }

    fn is_view_class(&self, id: HeapId) -> bool {
        class_is_view(self.vm.heap, self.vm.interns, id)
    }
}

fn args_to_host(vm: &mut VM<'_>, args: ArgValues) -> RunResult<Vec<HostValue>> {
    let pos = args.into_pos_only("host method", vm.heap)?;
    let mut out = Vec::new();
    for value in pos {
        out.push(value_to_host(vm, value)?);
    }
    Ok(out)
}

fn value_to_host(vm: &mut VM<'_>, value: Value) -> RunResult<HostValue> {
    match value {
        Value::None => Ok(HostValue::None),
        Value::Bool(b) => Ok(HostValue::Bool(b)),
        Value::Int(i) => Ok(HostValue::Int(i)),
        Value::Float(f) => Ok(HostValue::Float(f)),
        Value::InternString(id) => Ok(HostValue::Str(vm.interns.get_str(id).to_owned())),
        Value::Ref(id) => {
            let text = match vm.heap.get(id) {
                HeapData::Str(s) => Some(s.as_str().to_owned()),
                _ => None,
            };
            if let Some(text) = text {
                Value::Ref(id).drop_with(vm);
                Ok(HostValue::Str(text))
            } else {
                Ok(HostValue::Heap(id))
            }
        }
        other => {
            other.drop_with(vm);
            Err(SimpleException::new_msg(
                ExcType::TypeError,
                "unsupported value at the host boundary",
            )
            .into())
        }
    }
}

fn host_value_to_value(vm: &mut VM<'_>, value: HostValue) -> Value {
    match value {
        HostValue::None => Value::None,
        HostValue::Bool(b) => Value::Bool(b),
        HostValue::Int(i) => Value::Int(i),
        HostValue::Float(f) => Value::Float(f),
        HostValue::Str(s) => allocate_string(s, vm.heap),
        HostValue::Heap(id) => Value::Ref(id),
    }
}

fn host_values_to_args(vm: &mut VM<'_>, args: Vec<HostValue>) -> Result<ArgValues, MontyException> {
    let mut values: Vec<Value> = args.into_iter().map(|v| host_value_to_value(vm, v)).collect();
    Ok(match values.len() {
        0 => ArgValues::Empty,
        1 => ArgValues::One(values.pop().unwrap()),
        2 => {
            let b = values.pop().unwrap();
            let a = values.pop().unwrap();
            ArgValues::Two(a, b)
        }
        _ => ArgValues::ArgsKargs {
            args: values,
            kwargs: KwargsValues::Empty,
        },
    })
}

fn run_err_to_monty(err: RunError) -> MontyException {
    MontyException::runtime_error(format!("{err:?}"))
}

fn run_err_from_monty(err: MontyException) -> RunError {
    SimpleException::new_msg(ExcType::RuntimeError, err.to_string()).into()
}

impl<'h> PyTrait<'h> for HeapObjectRead<'h, HostObject> {
    fn py_type(&self, _vm: &VM<'h>) -> Type {
        Type::HostObject
    }

    fn py_len(&self, _vm: &VM<'h>) -> Option<usize> {
        None
    }

    fn py_call_attr(
        &mut self,
        vm: &mut VM<'h>,
        attr: &crate::value::EitherStr,
        args: ArgValues,
    ) -> RunResult<CallResult> {
        let id = self.id();
        let obj = *self.get(vm.heap);
        dispatch_call_attr(vm, id, obj, attr.as_str(vm.interns), args)
    }

    fn py_getattr(
        &self,
        attr: &crate::value::EitherStr,
        vm: &mut VM<'h>,
    ) -> RunResult<Option<CallResult>> {
        let id = self.id();
        let obj = *self.get(vm.heap);
        dispatch_getattr(vm, id, obj, attr.as_str(vm.interns))
    }

    fn py_eq_impl(&self, other: &Value, vm: &mut VM<'h>) -> RunResult<Option<bool>> {
        Ok(match other {
            Value::Ref(id) => match vm.heap.get(*id) {
                HeapData::HostObject(other_obj) => {
                    let this = self.get(vm.heap);
                    Some(this.kind == other_obj.kind && this.data == other_obj.data)
                }
                _ => None,
            },
            _ => None,
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::embed::HostVtable;

    struct NoHost;

    impl HostVtable for NoHost {
        fn call_attr(
            &mut self,
            _ctx: &mut dyn HostCtx,
            _id: HeapId,
            _obj: HostObject,
            attr: &str,
            _args: Vec<HostValue>,
        ) -> Result<HostValue, String> {
            Err(format!("no host method {attr}"))
        }

        fn getattr(
            &mut self,
            _ctx: &mut dyn HostCtx,
            _id: HeapId,
            _obj: HostObject,
            _attr: &str,
        ) -> Result<Option<HostValue>, String> {
            Ok(None)
        }

        fn construct(
            &mut self,
            _ctx: &mut dyn HostCtx,
            name: &str,
            _args: Vec<HostValue>,
        ) -> Result<HostValue, String> {
            Err(format!("no host constructor {name}"))
        }
    }

    #[test]
    fn from_ui_import_button_loads_host_module() {
        let extra = vec![HostModuleSource {
            name: "ui".into(),
            filename: "ui.py".into(),
            source: "X = 41\ndef button():\n    return X + 1\n".into(),
        }];
        let host: Rc<RefCell<dyn HostVtable>> = Rc::new(RefCell::new(NoHost));
        let mut embed = Embed::run_source_with_modules(
            "from ui import button\ndef check():\n    return button()\n".into(),
            "main.py",
            extra,
            host,
        )
        .expect("compile/run");
        match embed.call_global("check", vec![]).expect("check()") {
            HostValue::Int(42) => {}
            other => panic!("expected 42, got {other:?}"),
        }
    }

    #[test]
    fn unknown_host_module_is_module_not_found() {
        let host: Rc<RefCell<dyn HostVtable>> = Rc::new(RefCell::new(NoHost));
        let err = match Embed::run_source(
            "from missing_mod import x\n".into(),
            "main.py",
            host,
        ) {
            Ok(_) => panic!("missing module should fail"),
            Err(err) => err,
        };
        let msg = err.to_string();
        assert!(
            msg.contains("missing_mod") || msg.to_lowercase().contains("module"),
            "unexpected error: {msg}"
        );
    }
}
