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
    KIND_DOCK_AREA, KIND_DOCK_AREA_TYPE,
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
