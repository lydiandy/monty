//! Former host UI module. Discriminant and [`ModuleFunctions`] remain so dumps
//! keep their ids. Live `from ui import` is registered by the embedder.

use crate::{
    args::ArgValues,
    bytecode::VM,
    embed::{
        HostObject, KIND_ACCORDION_HEADER_TYPE, KIND_ACCORDION_ITEM_TYPE, KIND_ACCORDION_PANEL_TYPE,
        KIND_ACCORDION_TRIGGER_TYPE, KIND_ACCORDION_TYPE, KIND_ALERT_DIALOG_ACTION_TYPE,
        KIND_ALERT_DIALOG_BACKDROP_TYPE, KIND_ALERT_DIALOG_CANCEL_TYPE, KIND_ALERT_DIALOG_CLOSE_TYPE,
        KIND_ALERT_DIALOG_DESCRIPTION_TYPE, KIND_ALERT_DIALOG_POPUP_TYPE, KIND_ALERT_DIALOG_TITLE_TYPE,
        KIND_ALERT_DIALOG_TRIGGER_TYPE, KIND_ALERT_DIALOG_TYPE, KIND_AVATAR_FALLBACK_TYPE, KIND_AVATAR_IMAGE_TYPE,
        KIND_AVATAR_TYPE, KIND_BACKGROUND_TYPE, KIND_BUTTON_TYPE, KIND_CALENDAR_STATE_TYPE, KIND_CALENDAR_TYPE,
        KIND_CHECKBOX_TYPE, KIND_COLLAPSIBLE_TYPE, KIND_COLOR_PICKER_STATE_TYPE, KIND_COLOR_PICKER_TYPE,
        KIND_COLOR_SWATCH_TYPE, KIND_COMBOBOX_TYPE, KIND_DATE_PICKER_TYPE, KIND_DOCK_AREA_TYPE, KIND_FOCUS_HANDLE_TYPE,
        KIND_FS, KIND_HOVER_CARD_TYPE, KIND_HTTP, KIND_INPUT_STATE_TYPE, KIND_INPUT_TYPE, KIND_LINK_TYPE, KIND_NET,
        KIND_NUMBER_INPUT_TYPE, KIND_OTP_INPUT_TYPE, KIND_OTP_STATE_TYPE, KIND_PAGINATION_TYPE, KIND_PATH_BUILDER_TYPE,
        KIND_POPOVER_TYPE, KIND_POPUP_TYPE, KIND_PROCESS, KIND_PROGRESS_INDICATOR_TYPE, KIND_PROGRESS_TRACK_TYPE,
        KIND_PROGRESS_TYPE, KIND_RADIO_GROUP_TYPE, KIND_RADIO_TYPE, KIND_SCROLLBAR_TYPE, KIND_SELECT_TYPE,
        KIND_SLIDER_INDICATOR_TYPE, KIND_SLIDER_STATE_TYPE, KIND_SLIDER_THUMB_TYPE, KIND_SLIDER_TRACK_TYPE,
        KIND_SLIDER_TYPE, KIND_STORAGE, KIND_SWITCH_TYPE, KIND_TAB_TYPE, KIND_TABLE_BODY_TYPE, KIND_TABLE_CAPTION_TYPE,
        KIND_TABLE_CELL_TYPE, KIND_TABLE_HEAD_TYPE, KIND_TABLE_HEADER_TYPE, KIND_TABLE_ROW_TYPE, KIND_TABLE_TYPE,
        KIND_TABS_TYPE, KIND_TEXT_VIEW_TYPE, KIND_TEXTAREA_STATE_TYPE, KIND_TEXTAREA_TYPE, KIND_TOGGLE_GROUP_TYPE,
        KIND_TOGGLE_TYPE, KIND_TREE_STATE_TYPE, KIND_TREE_TYPE, KIND_WEBSOCKET, KIND_WINDOW,
    },
    exception_private::RunResult,
    heap::{HeapData, HeapId},
    modules::{ModuleFunctions, gpui, gpui_base},
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
    let mut module = Module::named(vm, "ui");
    module.set_attr_str(
        "view",
        Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Runtime(gpui::GpuiFunctions::View))),
        vm,
    );
    for (name, kind, data) in [
        ("window", KIND_WINDOW, 0),
        ("localStorage", KIND_STORAGE, 0),
        ("sessionStorage", KIND_STORAGE, 1),
        ("fs", KIND_FS, 0),
        ("process", KIND_PROCESS, 0),
        ("http", KIND_HTTP, 0),
        ("websocket", KIND_WEBSOCKET, 0),
        ("net", KIND_NET, 0),
    ] {
        let obj = vm.heap.allocate(HeapData::HostObject(HostObject { kind, data }));
        module.set_attr_str(name, Value::Ref(obj), vm);
    }
    for (name, function) in [
        ("v_flex", gpui_base::GpuiBaseFunctions::VFlex),
        ("h_flex", gpui_base::GpuiBaseFunctions::HFlex),
        ("div", gpui_base::GpuiBaseFunctions::Div),
        ("svg", gpui_base::GpuiBaseFunctions::Svg),
        ("image", gpui_base::GpuiBaseFunctions::Image),
        ("v_virtual_list", gpui_base::GpuiBaseFunctions::VVirtualList),
        ("h_virtual_list", gpui_base::GpuiBaseFunctions::HVirtualList),
        ("dock_area", gpui_base::GpuiBaseFunctions::DockArea),
        ("dock_content", gpui_base::GpuiBaseFunctions::DockContent),
        ("pagination_items", gpui_base::GpuiBaseFunctions::PaginationItems),
        ("h_resizable", gpui_base::GpuiBaseFunctions::HResizable),
        ("v_resizable", gpui_base::GpuiBaseFunctions::VResizable),
        ("resizable_panel", gpui_base::GpuiBaseFunctions::ResizablePanel),
        ("scene", gpui_base::GpuiBaseFunctions::Scene),
        ("node", gpui_base::GpuiBaseFunctions::Node),
        ("edge", gpui_base::GpuiBaseFunctions::Edge),
        ("play", gpui_base::GpuiBaseFunctions::Play),
        ("seq", gpui_base::GpuiBaseFunctions::Seq),
        ("fps_monitor", gpui_base::GpuiBaseFunctions::FpsMonitor),
    ] {
        module.set_attr_str(
            name,
            Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Widgets(function))),
            vm,
        );
    }
    for (name, function) in [
        ("set_theme", ThemeFunctions::SetTheme),
        ("list_themes", ThemeFunctions::ListThemes),
    ] {
        module.set_attr_str(
            name,
            Value::ModuleFunction(ModuleFunctions::Ui(UiFunctions::Theme(function))),
            vm,
        );
    }
    for (name, kind) in [
        ("Button", KIND_BUTTON_TYPE),
        ("Checkbox", KIND_CHECKBOX_TYPE),
        ("Switch", KIND_SWITCH_TYPE),
        ("Link", KIND_LINK_TYPE),
        ("InputState", KIND_INPUT_STATE_TYPE),
        ("TextareaState", KIND_TEXTAREA_STATE_TYPE),
        ("Input", KIND_INPUT_TYPE),
        ("Textarea", KIND_TEXTAREA_TYPE),
        ("DockArea", KIND_DOCK_AREA_TYPE),
        ("PathBuilder", KIND_PATH_BUILDER_TYPE),
        ("Background", KIND_BACKGROUND_TYPE),
        ("FocusHandle", KIND_FOCUS_HANDLE_TYPE),
        ("NumberInput", KIND_NUMBER_INPUT_TYPE),
        ("OtpInput", KIND_OTP_INPUT_TYPE),
        ("OtpState", KIND_OTP_STATE_TYPE),
        ("Slider", KIND_SLIDER_TYPE),
        ("SliderTrack", KIND_SLIDER_TRACK_TYPE),
        ("SliderIndicator", KIND_SLIDER_INDICATOR_TYPE),
        ("SliderThumb", KIND_SLIDER_THUMB_TYPE),
        ("SliderState", KIND_SLIDER_STATE_TYPE),
        ("Progress", KIND_PROGRESS_TYPE),
        ("ProgressTrack", KIND_PROGRESS_TRACK_TYPE),
        ("ProgressIndicator", KIND_PROGRESS_INDICATOR_TYPE),
        ("Avatar", KIND_AVATAR_TYPE),
        ("AvatarImage", KIND_AVATAR_IMAGE_TYPE),
        ("AvatarFallback", KIND_AVATAR_FALLBACK_TYPE),
        ("Pagination", KIND_PAGINATION_TYPE),
        ("Tabs", KIND_TABS_TYPE),
        ("Tab", KIND_TAB_TYPE),
        ("Accordion", KIND_ACCORDION_TYPE),
        ("AccordionItem", KIND_ACCORDION_ITEM_TYPE),
        ("AccordionHeader", KIND_ACCORDION_HEADER_TYPE),
        ("AccordionPanel", KIND_ACCORDION_PANEL_TYPE),
        ("AccordionTrigger", KIND_ACCORDION_TRIGGER_TYPE),
        ("Radio", KIND_RADIO_TYPE),
        ("RadioGroup", KIND_RADIO_GROUP_TYPE),
        ("Toggle", KIND_TOGGLE_TYPE),
        ("ToggleGroup", KIND_TOGGLE_GROUP_TYPE),
        ("Table", KIND_TABLE_TYPE),
        ("TableHeader", KIND_TABLE_HEADER_TYPE),
        ("TableBody", KIND_TABLE_BODY_TYPE),
        ("TableRow", KIND_TABLE_ROW_TYPE),
        ("TableHead", KIND_TABLE_HEAD_TYPE),
        ("TableCell", KIND_TABLE_CELL_TYPE),
        ("TableCaption", KIND_TABLE_CAPTION_TYPE),
        ("Collapsible", KIND_COLLAPSIBLE_TYPE),
        ("Popover", KIND_POPOVER_TYPE),
        ("HoverCard", KIND_HOVER_CARD_TYPE),
        ("Popup", KIND_POPUP_TYPE),
        ("Select", KIND_SELECT_TYPE),
        ("Combobox", KIND_COMBOBOX_TYPE),
        ("DatePicker", KIND_DATE_PICKER_TYPE),
        ("CalendarState", KIND_CALENDAR_STATE_TYPE),
        ("Calendar", KIND_CALENDAR_TYPE),
        ("ColorPicker", KIND_COLOR_PICKER_TYPE),
        ("ColorPickerState", KIND_COLOR_PICKER_STATE_TYPE),
        ("ColorSwatch", KIND_COLOR_SWATCH_TYPE),
        ("Tree", KIND_TREE_TYPE),
        ("TreeState", KIND_TREE_STATE_TYPE),
        ("AlertDialog", KIND_ALERT_DIALOG_TYPE),
        ("AlertDialogTrigger", KIND_ALERT_DIALOG_TRIGGER_TYPE),
        ("AlertDialogPopup", KIND_ALERT_DIALOG_POPUP_TYPE),
        ("AlertDialogTitle", KIND_ALERT_DIALOG_TITLE_TYPE),
        ("AlertDialogDescription", KIND_ALERT_DIALOG_DESCRIPTION_TYPE),
        ("AlertDialogAction", KIND_ALERT_DIALOG_ACTION_TYPE),
        ("AlertDialogCancel", KIND_ALERT_DIALOG_CANCEL_TYPE),
        ("AlertDialogBackdrop", KIND_ALERT_DIALOG_BACKDROP_TYPE),
        ("AlertDialogClose", KIND_ALERT_DIALOG_CLOSE_TYPE),
        ("Scrollbar", KIND_SCROLLBAR_TYPE),
        ("TextView", KIND_TEXT_VIEW_TYPE),
    ] {
        let ty = vm.heap.allocate(HeapData::HostObject(HostObject { kind, data: 0 }));
        module.set_attr_str(name, Value::Ref(ty), vm);
    }
    vm.heap.allocate(HeapData::Module(Box::new(module)))
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
