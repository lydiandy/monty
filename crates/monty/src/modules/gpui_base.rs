//! Former `gpui_base` widget constructors. Kept so dump
//! `ModuleFunctions::GpuiBase` still decodes. Scripts import `ui`.

use crate::{
    args::ArgValues,
    bytecode::VM,
    embed::{
        self, HostObject, KIND_BUTTON_TYPE, KIND_CHECKBOX_TYPE, KIND_DOCK_AREA_TYPE,
        KIND_INPUT_STATE_TYPE, KIND_INPUT_TYPE, KIND_LINK_TYPE, KIND_SWITCH_TYPE,
        KIND_TEXTAREA_STATE_TYPE, KIND_TEXTAREA_TYPE,
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
    #[strum(serialize = "svg")]
    Svg,
    #[strum(serialize = "image")]
    Image,
    #[strum(serialize = "v_virtual_list")]
    VVirtualList,
    #[strum(serialize = "h_virtual_list")]
    HVirtualList,
    #[strum(serialize = "dock_area")]
    DockArea,
    #[strum(serialize = "dock_content")]
    DockContent,
    #[strum(serialize = "pagination_items")]
    PaginationItems,
    #[strum(serialize = "h_resizable")]
    HResizable,
    #[strum(serialize = "v_resizable")]
    VResizable,
    #[strum(serialize = "resizable_panel")]
    ResizablePanel,
    #[strum(serialize = "scene")]
    Scene,
    #[strum(serialize = "node")]
    Node,
    #[strum(serialize = "edge")]
    Edge,
    #[strum(serialize = "play")]
    Play,
    #[strum(serialize = "seq")]
    Seq,
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
    module.set_attr(
        StaticStrings::Svg,
        Value::ModuleFunction(ModuleFunctions::GpuiBase(GpuiBaseFunctions::Svg)),
        vm,
    );
    module.set_attr(
        StaticStrings::Image,
        Value::ModuleFunction(ModuleFunctions::GpuiBase(GpuiBaseFunctions::Image)),
        vm,
    );
    module.set_attr(
        StaticStrings::VVirtualList,
        Value::ModuleFunction(ModuleFunctions::GpuiBase(GpuiBaseFunctions::VVirtualList)),
        vm,
    );
    module.set_attr(
        StaticStrings::HVirtualList,
        Value::ModuleFunction(ModuleFunctions::GpuiBase(GpuiBaseFunctions::HVirtualList)),
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
    let textarea_state_ty = vm.heap.allocate(HeapData::HostObject(HostObject {
        kind: KIND_TEXTAREA_STATE_TYPE,
        data: 0,
    }));
    module.set_attr(
        StaticStrings::TextareaState,
        Value::Ref(textarea_state_ty),
        vm,
    );
    let input_ty = vm.heap.allocate(HeapData::HostObject(HostObject {
        kind: KIND_INPUT_TYPE,
        data: 0,
    }));
    module.set_attr(StaticStrings::Input, Value::Ref(input_ty), vm);
    let textarea_ty = vm.heap.allocate(HeapData::HostObject(HostObject {
        kind: KIND_TEXTAREA_TYPE,
        data: 0,
    }));
    module.set_attr(StaticStrings::Textarea, Value::Ref(textarea_ty), vm);
    module.set_attr(
        StaticStrings::DockAreaFn,
        Value::ModuleFunction(ModuleFunctions::GpuiBase(GpuiBaseFunctions::DockArea)),
        vm,
    );
    module.set_attr(
        StaticStrings::DockContent,
        Value::ModuleFunction(ModuleFunctions::GpuiBase(GpuiBaseFunctions::DockContent)),
        vm,
    );
    let dock_area_ty = vm.heap.allocate(HeapData::HostObject(HostObject {
        kind: KIND_DOCK_AREA_TYPE,
        data: 0,
    }));
    module.set_attr(StaticStrings::DockArea, Value::Ref(dock_area_ty), vm);
    vm.heap.allocate(HeapData::Module(Box::new(module)))
}

pub(crate) fn call(vm: &mut VM<'_>, function: GpuiBaseFunctions, args: ArgValues) -> RunResult<Value> {
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
        GpuiBaseFunctions::Svg => embed::dispatch_construct(vm, "svg", args),
        GpuiBaseFunctions::Image => embed::dispatch_construct(vm, "image", args),
        GpuiBaseFunctions::VVirtualList => {
            embed::dispatch_construct(vm, "v_virtual_list", args)
        }
        GpuiBaseFunctions::HVirtualList => {
            embed::dispatch_construct(vm, "h_virtual_list", args)
        }
        GpuiBaseFunctions::DockArea => embed::dispatch_construct(vm, "dock_area", args),
        GpuiBaseFunctions::DockContent => {
            args.check_zero_args("dock_content", vm.heap)?;
            embed::dispatch_construct(vm, "dock_content", ArgValues::Empty)
        }
        GpuiBaseFunctions::PaginationItems => {
            embed::dispatch_construct(vm, "pagination_items", args)
        }
        GpuiBaseFunctions::HResizable => embed::dispatch_construct(vm, "h_resizable", args),
        GpuiBaseFunctions::VResizable => embed::dispatch_construct(vm, "v_resizable", args),
        GpuiBaseFunctions::ResizablePanel => {
            args.check_zero_args("resizable_panel", vm.heap)?;
            embed::dispatch_construct(vm, "resizable_panel", ArgValues::Empty)
        }
        GpuiBaseFunctions::Scene => embed::dispatch_construct(vm, "scene", args),
        GpuiBaseFunctions::Node => embed::dispatch_construct(vm, "node", args),
        GpuiBaseFunctions::Edge => embed::dispatch_construct(vm, "edge", args),
        GpuiBaseFunctions::Play => embed::dispatch_construct(vm, "play", args),
        GpuiBaseFunctions::Seq => embed::dispatch_construct(vm, "seq", args),
    }
}
