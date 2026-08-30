//! Host UI module (`from ui import view, v_flex, Button`).
//!
//! Combines the former `gpui` runtime exports and `gpui_base` widget
//! constructors. Old `Gpui` / `GpuiBase` [`ModuleFunctions`] variants remain
//! so dumps keep their discriminants.

use crate::{
    args::ArgValues,
    bytecode::VM,
    embed::{
        HostObject, KIND_ACCORDION_HEADER_TYPE, KIND_ACCORDION_ITEM_TYPE, KIND_ACCORDION_PANEL_TYPE,
        KIND_ACCORDION_TRIGGER_TYPE, KIND_ACCORDION_TYPE, KIND_AVATAR_FALLBACK_TYPE,
        KIND_AVATAR_IMAGE_TYPE, KIND_AVATAR_TYPE, KIND_BACKGROUND_TYPE, KIND_BUTTON_TYPE,
        KIND_CALENDAR_STATE_TYPE, KIND_CHECKBOX_TYPE, KIND_COLLAPSIBLE_TYPE, KIND_COMBOBOX_TYPE,
        KIND_DATE_PICKER_TYPE, KIND_DOCK_AREA_TYPE, KIND_FOCUS_HANDLE_TYPE, KIND_FS, KIND_NET,
        KIND_HOVER_CARD_TYPE, KIND_HTTP,
        KIND_INPUT_STATE_TYPE, KIND_INPUT_TYPE, KIND_LINK_TYPE, KIND_NUMBER_INPUT_TYPE,
        KIND_OTP_INPUT_TYPE, KIND_OTP_STATE_TYPE, KIND_PAGINATION_TYPE, KIND_PATH_BUILDER_TYPE,
        KIND_POPOVER_TYPE, KIND_POPUP_TYPE, KIND_PROCESS, KIND_PROGRESS_INDICATOR_TYPE,
        KIND_PROGRESS_TRACK_TYPE, KIND_PROGRESS_TYPE, KIND_RADIO_GROUP_TYPE, KIND_RADIO_TYPE,
        KIND_SCROLLBAR_TYPE, KIND_SELECT_TYPE, KIND_SLIDER_INDICATOR_TYPE, KIND_SLIDER_STATE_TYPE,
        KIND_SLIDER_THUMB_TYPE, KIND_SLIDER_TRACK_TYPE, KIND_SLIDER_TYPE, KIND_STORAGE,
        KIND_SWITCH_TYPE, KIND_TABLE_BODY_TYPE, KIND_TABLE_CAPTION_TYPE, KIND_TABLE_CELL_TYPE,
        KIND_TABLE_HEAD_TYPE, KIND_TABLE_HEADER_TYPE, KIND_TABLE_ROW_TYPE, KIND_TABLE_TYPE,
        KIND_TABS_TYPE, KIND_TAB_TYPE, KIND_TEXTAREA_STATE_TYPE, KIND_TEXTAREA_TYPE,
        KIND_TOGGLE_GROUP_TYPE, KIND_TOGGLE_TYPE, KIND_WEBSOCKET, KIND_WINDOW,
    },
    exception_private::RunResult,
    heap::{HeapData, HeapId},
    intern::StaticStrings,
    modules::{gpui, gpui_base, ModuleFunctions},
    types::Module,
    value::Value,
};

/// `ui` 主题函数。追加变体，不动 Runtime / Widgets 判别式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::Display, serde::Serialize, serde::Deserialize)]
pub(crate) enum ThemeFunctions {
    #[strum(serialize = "set_theme")]
    SetTheme,
    #[strum(serialize = "load_theme")]
    LoadTheme,
    #[strum(serialize = "list_themes")]
    ListThemes,
}

/// Functions exposed by the `ui` module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) enum UiFunctions {
    Runtime(gpui::GpuiFunctions),
    Widgets(gpui_base::GpuiBaseFunctions),
    Theme(ThemeFunctions),
}

impl std::fmt::Display for UiFunctions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(func) => write!(f, "{func}"),
            Self::Widgets(func) => write!(f, "{func}"),
            Self::Theme(func) => write!(f, "{func}"),
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
    let net = vm.heap.allocate(HeapData::HostObject(HostObject {
        kind: KIND_NET,
        data: 0,
    }));
    module.set_attr(StaticStrings::Net, Value::Ref(net), vm);

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
    module.set_attr(
        StaticStrings::SetTheme,
        Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Theme(
            ThemeFunctions::SetTheme,
        ))),
        vm,
    );
    module.set_attr(
        StaticStrings::LoadTheme,
        Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Theme(
            ThemeFunctions::LoadTheme,
        ))),
        vm,
    );
    module.set_attr(
        StaticStrings::ListThemes,
        Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Theme(
            ThemeFunctions::ListThemes,
        ))),
        vm,
    );
    let path_builder_ty = vm.heap.allocate(HeapData::HostObject(HostObject {
        kind: KIND_PATH_BUILDER_TYPE,
        data: 0,
    }));
    module.set_attr(StaticStrings::PathBuilder, Value::Ref(path_builder_ty), vm);
    let background_ty = vm.heap.allocate(HeapData::HostObject(HostObject {
        kind: KIND_BACKGROUND_TYPE,
        data: 0,
    }));
    module.set_attr(StaticStrings::Background, Value::Ref(background_ty), vm);
    export_type(
        vm,
        &mut module,
        StaticStrings::FocusHandle,
        KIND_FOCUS_HANDLE_TYPE,
    );
    export_type(vm, &mut module, StaticStrings::NumberInput, KIND_NUMBER_INPUT_TYPE);
    export_type(vm, &mut module, StaticStrings::OtpInput, KIND_OTP_INPUT_TYPE);
    export_type(vm, &mut module, StaticStrings::OtpState, KIND_OTP_STATE_TYPE);
    export_type(vm, &mut module, StaticStrings::Slider, KIND_SLIDER_TYPE);
    export_type(vm, &mut module, StaticStrings::SliderTrack, KIND_SLIDER_TRACK_TYPE);
    export_type(
        vm,
        &mut module,
        StaticStrings::SliderIndicator,
        KIND_SLIDER_INDICATOR_TYPE,
    );
    export_type(vm, &mut module, StaticStrings::SliderThumb, KIND_SLIDER_THUMB_TYPE);
    export_type(vm, &mut module, StaticStrings::SliderState, KIND_SLIDER_STATE_TYPE);
    export_type(vm, &mut module, StaticStrings::Progress, KIND_PROGRESS_TYPE);
    export_type(
        vm,
        &mut module,
        StaticStrings::ProgressTrack,
        KIND_PROGRESS_TRACK_TYPE,
    );
    export_type(
        vm,
        &mut module,
        StaticStrings::ProgressIndicator,
        KIND_PROGRESS_INDICATOR_TYPE,
    );
    export_type(vm, &mut module, StaticStrings::Avatar, KIND_AVATAR_TYPE);
    export_type(vm, &mut module, StaticStrings::AvatarImage, KIND_AVATAR_IMAGE_TYPE);
    export_type(
        vm,
        &mut module,
        StaticStrings::AvatarFallback,
        KIND_AVATAR_FALLBACK_TYPE,
    );
    export_type(vm, &mut module, StaticStrings::Pagination, KIND_PAGINATION_TYPE);
    export_type(vm, &mut module, StaticStrings::Tabs, KIND_TABS_TYPE);
    export_type(vm, &mut module, StaticStrings::Tab, KIND_TAB_TYPE);
    export_type(vm, &mut module, StaticStrings::Accordion, KIND_ACCORDION_TYPE);
    export_type(
        vm,
        &mut module,
        StaticStrings::AccordionItem,
        KIND_ACCORDION_ITEM_TYPE,
    );
    export_type(
        vm,
        &mut module,
        StaticStrings::AccordionHeader,
        KIND_ACCORDION_HEADER_TYPE,
    );
    export_type(
        vm,
        &mut module,
        StaticStrings::AccordionPanel,
        KIND_ACCORDION_PANEL_TYPE,
    );
    export_type(
        vm,
        &mut module,
        StaticStrings::AccordionTrigger,
        KIND_ACCORDION_TRIGGER_TYPE,
    );
    export_type(vm, &mut module, StaticStrings::Radio, KIND_RADIO_TYPE);
    export_type(vm, &mut module, StaticStrings::RadioGroup, KIND_RADIO_GROUP_TYPE);
    export_type(vm, &mut module, StaticStrings::Toggle, KIND_TOGGLE_TYPE);
    export_type(vm, &mut module, StaticStrings::ToggleGroup, KIND_TOGGLE_GROUP_TYPE);
    export_type(vm, &mut module, StaticStrings::Table, KIND_TABLE_TYPE);
    export_type(vm, &mut module, StaticStrings::TableHeader, KIND_TABLE_HEADER_TYPE);
    export_type(vm, &mut module, StaticStrings::TableBody, KIND_TABLE_BODY_TYPE);
    export_type(vm, &mut module, StaticStrings::TableRow, KIND_TABLE_ROW_TYPE);
    export_type(vm, &mut module, StaticStrings::TableHead, KIND_TABLE_HEAD_TYPE);
    export_type(vm, &mut module, StaticStrings::TableCell, KIND_TABLE_CELL_TYPE);
    export_type(vm, &mut module, StaticStrings::TableCaption, KIND_TABLE_CAPTION_TYPE);
    export_type(vm, &mut module, StaticStrings::Collapsible, KIND_COLLAPSIBLE_TYPE);
    export_type(vm, &mut module, StaticStrings::Popover, KIND_POPOVER_TYPE);
    export_type(vm, &mut module, StaticStrings::HoverCard, KIND_HOVER_CARD_TYPE);
    export_type(vm, &mut module, StaticStrings::Popup, KIND_POPUP_TYPE);
    export_type(vm, &mut module, StaticStrings::Select, KIND_SELECT_TYPE);
    export_type(vm, &mut module, StaticStrings::Combobox, KIND_COMBOBOX_TYPE);
    export_type(vm, &mut module, StaticStrings::DatePicker, KIND_DATE_PICKER_TYPE);
    export_type(
        vm,
        &mut module,
        StaticStrings::CalendarState,
        KIND_CALENDAR_STATE_TYPE,
    );
    export_type(vm, &mut module, StaticStrings::Scrollbar, KIND_SCROLLBAR_TYPE);
    module.set_attr(
        StaticStrings::PaginationItems,
        Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Widgets(
            gpui_base::GpuiBaseFunctions::PaginationItems,
        ))),
        vm,
    );
    module.set_attr(
        StaticStrings::HResizable,
        Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Widgets(
            gpui_base::GpuiBaseFunctions::HResizable,
        ))),
        vm,
    );
    module.set_attr(
        StaticStrings::VResizable,
        Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Widgets(
            gpui_base::GpuiBaseFunctions::VResizable,
        ))),
        vm,
    );
    module.set_attr(
        StaticStrings::ResizablePanel,
        Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Widgets(
            gpui_base::GpuiBaseFunctions::ResizablePanel,
        ))),
        vm,
    );
    module.set_attr(
        StaticStrings::Scene,
        Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Widgets(
            gpui_base::GpuiBaseFunctions::Scene,
        ))),
        vm,
    );
    module.set_attr(
        StaticStrings::Node,
        Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Widgets(
            gpui_base::GpuiBaseFunctions::Node,
        ))),
        vm,
    );
    module.set_attr(
        StaticStrings::Edge,
        Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Widgets(
            gpui_base::GpuiBaseFunctions::Edge,
        ))),
        vm,
    );
    module.set_attr(
        StaticStrings::Play,
        Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Widgets(
            gpui_base::GpuiBaseFunctions::Play,
        ))),
        vm,
    );
    module.set_attr(
        StaticStrings::Seq,
        Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Widgets(
            gpui_base::GpuiBaseFunctions::Seq,
        ))),
        vm,
    );
    vm.heap.allocate(HeapData::Module(Box::new(module)))
}

fn export_type(vm: &mut VM<'_>, module: &mut Module, name: StaticStrings, kind: u16) {
    let ty = vm.heap.allocate(HeapData::HostObject(HostObject { kind, data: 0 }));
    module.set_attr(name, Value::Ref(ty), vm);
}

pub(super) fn call(vm: &mut VM<'_>, function: UiFunctions, args: ArgValues) -> RunResult<Value> {
    match function {
        UiFunctions::Runtime(function) => gpui::call(vm, function, args),
        UiFunctions::Widgets(function) => gpui_base::call(vm, function, args),
        UiFunctions::Theme(function) => {
            let name = match function {
                ThemeFunctions::SetTheme => "set_theme",
                ThemeFunctions::LoadTheme => "load_theme",
                ThemeFunctions::ListThemes => "list_themes",
            };
            crate::embed::dispatch_construct(vm, name, args)
        }
    }
}
