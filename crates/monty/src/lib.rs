#![doc = include_str!("../README.md")]
// these files first because they include macros for the rest of the crate to use
mod heap;
mod heap_traits;

mod args;
mod asyncio;
mod builtins;
mod bytecode;
mod codecs;
mod dump_format;
mod embed;
mod exception_private;
mod expressions;
mod fstring;
mod function;
mod hash;
mod heap_data;
mod host_modules;
mod identity;
mod intern;
mod modules;
mod name_map;
mod namespace;
mod object_bridge;
mod os_dispatch;
mod parse;
mod predicate;
mod prepare;
mod repl;
mod resource_checks;
mod run;
mod run_progress;
mod sorting;
mod source_map;
mod string_builder;
mod stringize;
mod types;
mod value;

pub use crate::embed::{
    Embed, HostCtx, HostModuleSource, HostObject, HostValue, HostVtable, KIND_ACCORDION_HEADER_TYPE,
    KIND_ACCORDION_ITEM_TYPE, KIND_ACCORDION_PANEL_TYPE, KIND_ACCORDION_TRIGGER_TYPE, KIND_ACCORDION_TYPE,
    KIND_ALERT_DIALOG_ACTION_TYPE, KIND_ALERT_DIALOG_BACKDROP_TYPE, KIND_ALERT_DIALOG_CANCEL_TYPE,
    KIND_ALERT_DIALOG_CLOSE_TYPE, KIND_ALERT_DIALOG_DESCRIPTION_TYPE, KIND_ALERT_DIALOG_POPUP_TYPE,
    KIND_ALERT_DIALOG_TITLE_TYPE, KIND_ALERT_DIALOG_TRIGGER_TYPE, KIND_ALERT_DIALOG_TYPE, KIND_AVATAR_FALLBACK_TYPE,
    KIND_AVATAR_IMAGE_TYPE, KIND_AVATAR_TYPE, KIND_BACKGROUND_TYPE, KIND_BUTTON_TYPE, KIND_CALENDAR_STATE,
    KIND_CALENDAR_STATE_TYPE, KIND_CALENDAR_TYPE, KIND_CHECKBOX_TYPE, KIND_COLLAPSIBLE_TYPE, KIND_COLOR_PICKER_STATE,
    KIND_COLOR_PICKER_STATE_TYPE, KIND_COLOR_PICKER_TYPE, KIND_COLOR_SWATCH_TYPE, KIND_COLORS, KIND_COMBOBOX_TYPE,
    KIND_CX, KIND_DATE_PICKER_TYPE, KIND_DOCK_AREA, KIND_DOCK_AREA_TYPE, KIND_ELEMENT, KIND_FOCUS_HANDLE_TYPE, KIND_FS,
    KIND_HOVER_CARD_TYPE, KIND_HTTP, KIND_INPUT_STATE, KIND_INPUT_STATE_TYPE, KIND_INPUT_TYPE, KIND_LINK_TYPE,
    KIND_NET, KIND_NUMBER_INPUT_TYPE, KIND_OTP_INPUT_TYPE, KIND_OTP_STATE, KIND_OTP_STATE_TYPE, KIND_PAGINATION_TYPE,
    KIND_PATH_BUILDER_TYPE, KIND_POPOVER_TYPE, KIND_POPUP_TYPE, KIND_PROCESS, KIND_PROGRESS_INDICATOR_TYPE,
    KIND_PROGRESS_TRACK_TYPE, KIND_PROGRESS_TYPE, KIND_RADIO_GROUP_TYPE, KIND_RADIO_TYPE, KIND_SCROLLBAR_TYPE,
    KIND_SELECT_TYPE, KIND_SLIDER_INDICATOR_TYPE, KIND_SLIDER_STATE, KIND_SLIDER_STATE_TYPE, KIND_SLIDER_THUMB_TYPE,
    KIND_SLIDER_TRACK_TYPE, KIND_SLIDER_TYPE, KIND_STORAGE, KIND_SWITCH_TYPE, KIND_TAB_TYPE, KIND_TABLE_BODY_TYPE,
    KIND_TABLE_CAPTION_TYPE, KIND_TABLE_CELL_TYPE, KIND_TABLE_HEAD_TYPE, KIND_TABLE_HEADER_TYPE, KIND_TABLE_ROW_TYPE,
    KIND_TABLE_TYPE, KIND_TABS_TYPE, KIND_TASK, KIND_TCP_SOCKET, KIND_TEXT_VIEW_TYPE, KIND_TEXTAREA_STATE,
    KIND_TEXTAREA_STATE_TYPE, KIND_TEXTAREA_TYPE, KIND_THEME, KIND_TIMER, KIND_TOGGLE_GROUP_TYPE, KIND_TOGGLE_TYPE,
    KIND_TREE_STATE, KIND_TREE_STATE_TYPE, KIND_TREE_TYPE, KIND_TURSO_CONN, KIND_TURSO_ROWS, KIND_DB_QUERY,
    KIND_DB_TABLE, KIND_VIEW, KIND_WEBSOCKET,
    KIND_WINDOW, KIND_WS_SOCKET,
};
#[cfg(feature = "test-hooks")]
#[doc(hidden)]
pub use crate::function::FunctionMetadataFault;
pub use crate::heap::HeapId;
#[cfg(feature = "ref-count-return")]
pub use crate::run::RefCountOutput;
pub use crate::{
    dump_format::{DUMP_VERSION, Dump, DumpError, Session, SessionRef, dump},
    repl::{
        MontyRepl, ReplContinuationMode, ReplFunctionCall, ReplNameLookup, ReplOsCall, ReplProgress,
        ReplResolveFutures, ReplStartError, detect_repl_continuation_mode,
    },
    run::MontyRun,
    run_progress::{FunctionCall, NameLookup, OsCall, ResolveFutures, RunProgress},
};
