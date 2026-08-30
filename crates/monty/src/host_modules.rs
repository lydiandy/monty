//! Host-registered application modules (`from widgets import button`).
//!
//! Monty's compiler only `LoadModule`s [`crate::modules::StandardLib`]. Unknown
//! names used to become `RaiseImportError` with no `LoadAttr`. Embedders register
//! an app directory's top-level `.py` files here; the compiler emits
//! `LoadHostModule` and the VM executes each file as its own module (shared intern
//! pool, own globals/`Code`). Not file concatenation.

use std::{cell::RefCell, collections::HashMap, sync::Arc};

use crate::{
    bytecode::Code,
    heap::HeapId,
    intern::FunctionId,
    name_map::NameMap,
};

/// Source for one top-level host module (`widgets.py` → name `"widgets"`).
#[derive(Debug, Clone)]
pub struct HostModuleSource {
    /// Import name (`widgets` in `from widgets import button`).
    pub name: String,
    /// Path used in tracebacks.
    pub filename: String,
    /// Module source.
    pub source: String,
}

/// Compiled host module, intern-pool-shared with the entry file.
#[derive(Debug, Clone)]
pub(crate) struct HostModuleSpec {
    pub code: Arc<Code>,
    pub names: NameMap,
    pub func_start: u32,
    pub func_end: u32,
}

/// Runtime cache: one module object per name, `sys.modules`-style.
#[derive(Debug, Clone, Copy)]
pub(crate) struct HostModuleRuntime {
    pub module_id: HeapId,
    /// `false` while the module body is still running (circular import sees a
    /// partial module).
    pub loaded: bool,
}

/// Compiled host modules plus the live module-object cache.
///
/// Stored on [`crate::run::Executor`] (`serde(skip)` — dumps are unused on the
/// Embed path) and borrowed by the VM for `'h`.
#[derive(Debug, Clone, Default)]
pub(crate) struct HostModuleRegistry {
    specs: HashMap<String, HostModuleSpec>,
    runtime: RefCell<HashMap<String, HostModuleRuntime>>,
}

impl HostModuleRegistry {
    pub fn new(specs: HashMap<String, HostModuleSpec>) -> Self {
        Self {
            specs,
            runtime: RefCell::new(HashMap::new()),
        }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.specs.is_empty()
    }

    pub fn get(&self, name: &str) -> Option<&HostModuleSpec> {
        self.specs.get(name)
    }

    pub fn cached(&self, name: &str) -> Option<HostModuleRuntime> {
        self.runtime.borrow().get(name).copied()
    }

    /// Heap ids of host modules whose body has finished running.
    pub fn loaded_module_ids(&self) -> Vec<HeapId> {
        self.runtime
            .borrow()
            .values()
            .filter(|rt| rt.loaded)
            .map(|rt| rt.module_id)
            .collect()
    }

    pub fn begin_load(&self, name: &str, module_id: HeapId) {
        self.runtime.borrow_mut().insert(
            name.to_owned(),
            HostModuleRuntime {
                module_id,
                loaded: false,
            },
        );
    }

    pub fn finish_load(&self, name: &str) {
        if let Some(rt) = self.runtime.borrow_mut().get_mut(name) {
            rt.loaded = true;
        }
    }

    /// A fully-executed host module that owns `id`, if any.
    pub fn loaded_for_function(&self, id: FunctionId) -> Option<(HeapId, &HostModuleSpec)> {
        let idx = u32::try_from(id.index()).ok()?;
        for (name, spec) in &self.specs {
            if idx >= spec.func_start && idx < spec.func_end {
                let rt = self.runtime.borrow().get(name).copied()?;
                if rt.loaded {
                    return Some((rt.module_id, spec));
                }
                return None;
            }
        }
        None
    }
}
