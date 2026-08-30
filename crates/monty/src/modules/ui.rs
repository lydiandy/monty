//! Host UI module (`from ui import view, v_flex, Button`).
//!
//! Combines the former `gpui` runtime exports and `gpui_base` widget
//! constructors. Old `Gpui` / `GpuiBase` [`ModuleFunctions`] variants remain
//! so dumps keep their discriminants.

use crate::{
    args::ArgValues,
    bytecode::VM,
    embed::{
        HostObject, KIND_BUTTON_TYPE, KIND_CHECKBOX_TYPE, KIND_DOCK_AREA_TYPE, KIND_FS, KIND_HTTP,
        KIND_INPUT_STATE_TYPE, KIND_INPUT_TYPE, KIND_LINK_TYPE, KIND_PROCESS, KIND_STORAGE,
        KIND_SWITCH_TYPE, KIND_TEXTAREA_STATE_TYPE, KIND_TEXTAREA_TYPE, KIND_WEBSOCKET, KIND_WINDOW,
    },
    exception_private::RunResult,
    heap::{HeapData, HeapId},
    intern::StaticStrings,
    modules::{gpui, gpui_base, ModuleFunctions},
    types::Module,
    value::Value,
};

/// Functions exposed by the `ui` module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) enum UiFunctions {
    Runtime(gpui::GpuiFunctions),
    Widgets(gpui_base::GpuiBaseFunctions),
}

impl std::fmt::Display for UiFunctions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(func) => write!(f, "{func}"),
            Self::Widgets(func) => write!(f, "{func}"),
        }
    }
}

/// Creates the `ui` module.
pub fn create_module(vm: &mut VM<'_>) -> HeapId {
    let mut module = Module::new(StaticStrings::Ui);
    module.set_attr(
        StaticStrings::View,
        Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Runtime(
            gpui::GpuiFunctions::View,
        ))),
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
    module.set_attr(
        StaticStrings::SessionStorage,
        Value::Ref(session_storage),
        vm,
    );
    let fs = vm.heap.allocate(HeapData::HostObject(HostObject {
        kind: KIND_FS,
        data: 0,
    }));
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

    module.set_attr(
        StaticStrings::VFlex,
        Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Widgets(
            gpui_base::GpuiBaseFunctions::VFlex,
        ))),
        vm,
    );
    module.set_attr(
        StaticStrings::HFlex,
        Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Widgets(
            gpui_base::GpuiBaseFunctions::HFlex,
        ))),
        vm,
    );
    module.set_attr(
        StaticStrings::Div,
        Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Widgets(
            gpui_base::GpuiBaseFunctions::Div,
        ))),
        vm,
    );
    module.set_attr(
        StaticStrings::Svg,
        Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Widgets(
            gpui_base::GpuiBaseFunctions::Svg,
        ))),
        vm,
    );
    module.set_attr(
        StaticStrings::Image,
        Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Widgets(
            gpui_base::GpuiBaseFunctions::Image,
        ))),
        vm,
    );
    module.set_attr(
        StaticStrings::VVirtualList,
        Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Widgets(
            gpui_base::GpuiBaseFunctions::VVirtualList,
        ))),
        vm,
    );
    module.set_attr(
        StaticStrings::HVirtualList,
        Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Widgets(
            gpui_base::GpuiBaseFunctions::HVirtualList,
        ))),
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
        Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Widgets(
            gpui_base::GpuiBaseFunctions::DockArea,
        ))),
        vm,
    );
    module.set_attr(
        StaticStrings::DockContent,
        Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Widgets(
            gpui_base::GpuiBaseFunctions::DockContent,
        ))),
        vm,
    );
    let dock_area_ty = vm.heap.allocate(HeapData::HostObject(HostObject {
        kind: KIND_DOCK_AREA_TYPE,
        data: 0,
    }));
    module.set_attr(StaticStrings::DockArea, Value::Ref(dock_area_ty), vm);
    vm.heap.allocate(HeapData::Module(Box::new(module)))
}

pub(super) fn call(vm: &mut VM<'_>, function: UiFunctions, args: ArgValues) -> RunResult<Value> {
    match function {
        UiFunctions::Runtime(function) => gpui::call(vm, function, args),
        UiFunctions::Widgets(function) => gpui_base::call(vm, function, args),
    }
}
