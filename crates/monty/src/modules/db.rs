//! `db` package (`from db import turso`).
//!
//! `turso` is a host object (`KIND_TURSO_CONN`, `data == 0`). Connections,
//! `KIND_DB_QUERY` and `KIND_DB_TABLE` chains run in monty-ui. This crate only
//! hangs the module and KIND. Do not export `orm`.

use crate::{
    bytecode::VM,
    embed::{HostObject, KIND_TURSO_CONN},
    heap::{HeapData, HeapId},
    intern::StaticStrings,
    types::Module,
    value::Value,
};

/// Creates the `db` module with a `turso` submodule object.
pub fn create_module(vm: &mut VM<'_>) -> HeapId {
    let mut module = Module::new(StaticStrings::Db);
    let turso = vm.heap.allocate(HeapData::HostObject(HostObject {
        kind: KIND_TURSO_CONN,
        data: 0,
    }));
    module.set_attr(StaticStrings::Turso, Value::Ref(turso), vm);
    vm.heap.allocate(HeapData::Module(Box::new(module)))
}
