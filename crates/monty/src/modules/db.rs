//! Former `db` package. Discriminant and KIND numbers remain so dumps keep
//! their ids. Live `from db import` is registered by the embedder.

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
