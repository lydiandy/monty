//! Host widget constructors (`from gpui_base import v_flex, h_flex, Button`).

use crate::{
    args::ArgValues,
    bytecode::VM,
    embed::{
        self, HostObject, KIND_BUTTON_TYPE, KIND_CHECKBOX_TYPE, KIND_INPUT_STATE_TYPE,
        KIND_LINK_TYPE, KIND_SWITCH_TYPE,
    },
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
    #[strum(serialize = "h_flex")]
    HFlex,
    #[strum(serialize = "div")]
    Div,
}

/// Creates the `gpui_base` module.
pub fn create_module(vm: &mut VM<'_>) -> HeapId {
    let mut module = Module::new(StaticStrings::GpuiBase);
    module.set_attr(
        StaticStrings::VFlex,
        Value::ModuleFunction(ModuleFunctions::GpuiBase(GpuiBaseFunctions::VFlex)),
        vm,
    );
    module.set_attr(
        StaticStrings::HFlex,
        Value::ModuleFunction(ModuleFunctions::GpuiBase(GpuiBaseFunctions::HFlex)),
        vm,
    );
    module.set_attr(
        StaticStrings::Div,
        Value::ModuleFunction(ModuleFunctions::GpuiBase(GpuiBaseFunctions::Div)),
        vm,
    );
    let button_ty = vm.heap.allocate(HeapData::HostObject(HostObject {
        kind: KIND_BUTTON_TYPE,
        data: 0,
    }));
    module.set_attr(StaticStrings::Button, Value::Ref(button_ty), vm);
    let checkbox_ty = vm.heap.allocate(HeapData::HostObject(HostObject {
        kind: KIND_CHECKBOX_TYPE,
        data: 0,
    }));
    module.set_attr(StaticStrings::Checkbox, Value::Ref(checkbox_ty), vm);
    let switch_ty = vm.heap.allocate(HeapData::HostObject(HostObject {
        kind: KIND_SWITCH_TYPE,
        data: 0,
    }));
    module.set_attr(StaticStrings::Switch, Value::Ref(switch_ty), vm);
    let link_ty = vm.heap.allocate(HeapData::HostObject(HostObject {
        kind: KIND_LINK_TYPE,
        data: 0,
    }));
    module.set_attr(StaticStrings::Link, Value::Ref(link_ty), vm);
    let input_state_ty = vm.heap.allocate(HeapData::HostObject(HostObject {
        kind: KIND_INPUT_STATE_TYPE,
        data: 0,
    }));
    module.set_attr(StaticStrings::InputState, Value::Ref(input_state_ty), vm);
    vm.heap.allocate(HeapData::Module(Box::new(module)))
}

pub(super) fn call(vm: &mut VM<'_>, function: GpuiBaseFunctions, args: ArgValues) -> RunResult<Value> {
    match function {
        GpuiBaseFunctions::VFlex => {
            args.check_zero_args("v_flex", vm.heap)?;
            embed::dispatch_construct(vm, "v_flex", ArgValues::Empty)
        }
        GpuiBaseFunctions::HFlex => {
            args.check_zero_args("h_flex", vm.heap)?;
            embed::dispatch_construct(vm, "h_flex", ArgValues::Empty)
        }
        GpuiBaseFunctions::Div => {
            args.check_zero_args("div", vm.heap)?;
            embed::dispatch_construct(vm, "div", ArgValues::Empty)
        }
    }
}
