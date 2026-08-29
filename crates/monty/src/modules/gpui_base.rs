//! Host widget constructors (`from gpui_base import v_flex, Button`).

use crate::{
    args::ArgValues,
    bytecode::VM,
    embed::{self, HostObject, KIND_BUTTON_TYPE},
    exception_private::RunResult,
    heap::{HeapData, HeapId},
    intern::StaticStrings,
    modules::ModuleFunctions,
    types::Module,
    value::Value,
};

/// Functions exposed by the `gpui_base` module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::Display, serde::Serialize, serde::Deserialize)]
pub(crate) enum GpuiBaseFunctions {
    #[strum(serialize = "v_flex")]
    VFlex,
}

/// Creates the `gpui_base` module.
pub fn create_module(vm: &mut VM<'_>) -> HeapId {
    let mut module = Module::new(StaticStrings::GpuiBase);
    module.set_attr(
        StaticStrings::VFlex,
        Value::ModuleFunction(ModuleFunctions::GpuiBase(GpuiBaseFunctions::VFlex)),
        vm,
    );
    let button_ty = vm.heap.allocate(HeapData::HostObject(HostObject {
        kind: KIND_BUTTON_TYPE,
        data: 0,
    }));
    module.set_attr(StaticStrings::Button, Value::Ref(button_ty), vm);
    vm.heap.allocate(HeapData::Module(Box::new(module)))
}

pub(super) fn call(vm: &mut VM<'_>, function: GpuiBaseFunctions, args: ArgValues) -> RunResult<Value> {
    match function {
        GpuiBaseFunctions::VFlex => {
            args.check_zero_args("v_flex", vm.heap)?;
            embed::dispatch_construct(vm, "v_flex", ArgValues::Empty)
        }
    }
}
