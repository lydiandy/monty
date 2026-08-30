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

#[cfg(feature = "test-hooks")]
#[doc(hidden)]
pub use crate::function::FunctionMetadataFault;
#[cfg(feature = "ref-count-return")]
pub use crate::run::RefCountOutput;
pub use crate::embed::{
    Embed, HostCtx, HostModuleSource, HostObject, HostValue, HostVtable, KIND_BUTTON_TYPE,
    KIND_CHECKBOX_TYPE, KIND_COLORS, KIND_CX, KIND_ELEMENT, KIND_FS, KIND_HTTP, KIND_INPUT_STATE,
    KIND_INPUT_STATE_TYPE, KIND_INPUT_TYPE, KIND_LINK_TYPE, KIND_PROCESS, KIND_SWITCH_TYPE,
    KIND_TEXTAREA_STATE, KIND_TEXTAREA_STATE_TYPE, KIND_TEXTAREA_TYPE, KIND_THEME,
    KIND_STORAGE, KIND_TASK, KIND_TIMER, KIND_VIEW, KIND_WEBSOCKET, KIND_WINDOW, KIND_WS_SOCKET,
    KIND_DOCK_AREA, KIND_DOCK_AREA_TYPE, KIND_PATH_BUILDER_TYPE, KIND_BACKGROUND_TYPE,
    KIND_NUMBER_INPUT_TYPE, KIND_OTP_INPUT_TYPE, KIND_OTP_STATE_TYPE, KIND_OTP_STATE,
    KIND_SLIDER_TYPE, KIND_SLIDER_TRACK_TYPE, KIND_SLIDER_INDICATOR_TYPE, KIND_SLIDER_THUMB_TYPE,
    KIND_SLIDER_STATE_TYPE, KIND_SLIDER_STATE, KIND_PROGRESS_TYPE, KIND_PROGRESS_TRACK_TYPE,
    KIND_PROGRESS_INDICATOR_TYPE, KIND_AVATAR_TYPE, KIND_AVATAR_IMAGE_TYPE, KIND_AVATAR_FALLBACK_TYPE,
    KIND_PAGINATION_TYPE, KIND_TABS_TYPE, KIND_TAB_TYPE, KIND_ACCORDION_TYPE,
    KIND_ACCORDION_ITEM_TYPE, KIND_ACCORDION_HEADER_TYPE, KIND_ACCORDION_PANEL_TYPE,
    KIND_ACCORDION_TRIGGER_TYPE, KIND_RADIO_TYPE, KIND_RADIO_GROUP_TYPE, KIND_TOGGLE_TYPE,
    KIND_TOGGLE_GROUP_TYPE, KIND_TABLE_TYPE, KIND_TABLE_HEADER_TYPE, KIND_TABLE_BODY_TYPE,
    KIND_TABLE_ROW_TYPE, KIND_TABLE_HEAD_TYPE, KIND_TABLE_CELL_TYPE, KIND_TABLE_CAPTION_TYPE,
    KIND_COLLAPSIBLE_TYPE, KIND_POPOVER_TYPE, KIND_HOVER_CARD_TYPE, KIND_POPUP_TYPE,
    KIND_SELECT_TYPE, KIND_COMBOBOX_TYPE, KIND_DATE_PICKER_TYPE, KIND_CALENDAR_STATE_TYPE,
    KIND_CALENDAR_STATE, KIND_SCROLLBAR_TYPE, KIND_FOCUS_HANDLE_TYPE, KIND_NET, KIND_TCP_SOCKET,
};
pub use crate::heap::HeapId;
pub use crate::{

    dump_format::{DUMP_VERSION, Dump, DumpError, Session, SessionRef, dump},
    repl::{
        MontyRepl, ReplContinuationMode, ReplFunctionCall, ReplNameLookup, ReplOsCall, ReplProgress,
        ReplResolveFutures, ReplStartError, detect_repl_continuation_mode,
    },
    run::MontyRun,
    run_progress::{FunctionCall, NameLookup, OsCall, ResolveFutures, RunProgress},
};
