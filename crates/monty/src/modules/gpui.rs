//! Former `gpui` runtime module. Kept so dump `ModuleFunctions::Gpui` still
//! decodes. Scripts import `ui` (`from ui import view`).
//!
//! `@view` marks a class as the application entry. Monty has no user-class
//! `__getattr__` and no inheritance, so the decorator sets `__gpui_view__`
//! via `setattr` (which `Class` implements).

use crate::{
    args::ArgValues,
    bytecode::VM,
    embed::{HostObject, KIND_FS, KIND_HTTP, KIND_PROCESS, KIND_STORAGE, KIND_WEBSOCKET, KIND_WINDOW},
    exception_private::{ExcType, ExcTypeExt, RunResult},
    heap::{HeapData, HeapId, HeapReadOutput},
    intern::StaticStrings,
    modules::ModuleFunctions,
    types::{Module, PyTrait},
    value::{EitherStr, Value},
};

/// Functions exposed by the `gpui` module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::Display, serde::Serialize, serde::Deserialize)]
pub(crate) enum GpuiFunctions {
    #[strum(serialize = "view")]
    View,
}

/// Creates the `gpui` module.
pub fn create_module(vm: &mut VM<'_>) -> HeapId {
    let mut module = Module::new(StaticStrings::Gpui);
    module.set_attr(
        StaticStrings::View,
        Value::ModuleFunction(ModuleFunctions::Gpui(GpuiFunctions::View)),
        vm,
    );
    let window = vm.heap.allocate(HeapData::HostObject(HostObject {
        kind: KIND_WINDOW,
        data: 0,
    }));
    module.set_attr(StaticStrings::Window, Value::Ref(window), vm);
    let local_storage = vm.heap.allocate(HeapData::HostObject(HostObject {
        kind: KIND_STORAGE,
        data: 0,
    }));
    module.set_attr(StaticStrings::LocalStorage, Value::Ref(local_storage), vm);
    let session_storage = vm.heap.allocate(HeapData::HostObject(HostObject {
        kind: KIND_STORAGE,
        data: 1,
    }));
    module.set_attr(StaticStrings::SessionStorage, Value::Ref(session_storage), vm);
    let fs = vm
        .heap
        .allocate(HeapData::HostObject(HostObject { kind: KIND_FS, data: 0 }));
    module.set_attr(StaticStrings::Fs, Value::Ref(fs), vm);
    let process = vm.heap.allocate(HeapData::HostObject(HostObject {
        kind: KIND_PROCESS,
        data: 0,
    }));
    module.set_attr(StaticStrings::Process, Value::Ref(process), vm);
    let http = vm.heap.allocate(HeapData::HostObject(HostObject {
        kind: KIND_HTTP,
        data: 0,
    }));
    module.set_attr(StaticStrings::Http, Value::Ref(http), vm);
    let websocket = vm.heap.allocate(HeapData::HostObject(HostObject {
        kind: KIND_WEBSOCKET,
        data: 0,
    }));
    module.set_attr(StaticStrings::Websocket, Value::Ref(websocket), vm);
    vm.heap.allocate(HeapData::Module(Box::new(module)))
}

pub(crate) fn call(vm: &mut VM<'_>, function: GpuiFunctions, args: ArgValues) -> RunResult<Value> {
    match function {
        GpuiFunctions::View => view(vm, args),
    }
}

/// `@view` — `setattr(cls, '__gpui_view__', True); return cls`.
fn view(vm: &mut VM<'_>, args: ArgValues) -> RunResult<Value> {
    let cls = args.get_one_arg("view", vm.heap)?;
    let Value::Ref(id) = cls else {
        cls.drop_with(vm);
        return Err(ExcType::type_error("@view expects a class"));
    };
    match vm.heap.read(id) {
        HeapReadOutput::Class(mut class) => {
            class.py_set_attr(&EitherStr::from(StaticStrings::GpuiView), Value::Bool(true), vm)?;
        }
        _ => {
            Value::Ref(id).drop_with(vm);
            return Err(ExcType::type_error("@view expects a class"));
        }
    }
    Ok(Value::Ref(id))
}
