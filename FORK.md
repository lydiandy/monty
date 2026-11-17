# lydiandy/monty fork

本仓库从 [pydantic/monty](https://github.com/pydantic/monty) fork 而来，给 monty-ui / monty-reqwest / monty-db / monty-typesafe 用。
2026-11-17 起只保留 `main` 一条主干：它基于上游 `f1c56fb4`（1.6.0，含类继承与 pyo3 0.30），加上下面这几处 fork 补丁。

## 本工作区的 remote

| remote | URL | 角色 |
| --- | --- | --- |
| `origin` | `https://github.com/lydiandy/monty.git` | 本 fork |
| `upstream` | `https://github.com/pydantic/monty.git` | 上游 |

同步上游：`git fetch upstream && git merge upstream/main`，冲突按下面「共享文件」处理。

## fork 加了什么

- `crates/monty-host` — 宿主值 API：`Arena`、`HostValue`（`Value`）、`HostObject`、`HeapId`、`Callback`、`Dispatch`、`HostCtx`。
  monty-ui、monty-reqwest、monty-db、monty-typesafe 都直接依赖它，不再各自对着 `MontyObject` 手写。
  它只有 `monty-types` 依赖，不反向依赖 `monty`。
- 沙箱类句柄 — `MontyNode::SandboxClass` / `MontyObject::sandbox_class` / `as_sandbox_class`，`object_bridge` 把
  `HeapReadOutput::Class` 导出成边界 uuid，也能经 `callback_value` 传回解释器。`@view` 类、宿主返回的类都靠它跨边界。
- 宿主类迭代可挂起 — `GET_ITER` 对 `HostClass` 发 `CallResult::MethodCall("__iter__")`，`FOR_ITER` 发
  `CallResult::AttrLookup("__next__")` + `PendingLookupEffect::HostNext`；恢复时有值就压栈当循环变量，`Undefined`
  就弹掉迭代器并按 `FOR_ITER` 的 offset 跳过后面的循环体。`PendingLookupEffect::apply` 因此返回 `Option<Value>`。
  这是 reqwest `iter_bytes` / `iter_lines` / `iter_sse` 在 1.6 里还能逐条流式的原因。
  `list(host_iter)` / `next(host_iter)` 仍走同步收集，不支持。
- 沙箱回调句柄 — 闭包、带默认值的函数与绑定方法跨边界为 `SandboxFunction`，宿主可以再传回去调用。

## 共享文件（冲突预算）

| 文件 | 为什么要动 |
| --- | --- |
| `crates/monty/src/object_bridge.rs` | 导出/导入 `SandboxClass`；`SandboxFunction` 的三种 heap 形态 |
| `crates/monty/src/bytecode/vm/mod.rs` | `GET_ITER` / `FOR_ITER` 的 `HostClass` 分支 |
| `crates/monty/src/bytecode/vm/attr.rs` | `PendingLookupEffect::HostNext` 与 `apply` 返回 `Option<Value>` |
| `crates/monty/src/bytecode/vm/call.rs`、`crates/monty/src/repl.rs`、`crates/monty/src/run_progress.rs` | 跟着 `apply` 的新返回类型处理「不压栈」 |
| `crates/monty-types/src/graph.rs`、`object.rs` | `SandboxClass` 节点、构造与 Display |
| `crates/monty/tests/sandbox_callback.rs` | 类句柄往返、函数/闭包/绑定方法回调 |

## 分支

只有 `main`。旧的 `fork-1.6` / `fork-main` / `nsh-1.6` 不再维护；旧 v1.4 的历史留在 `fork-main` 的远端引用里。
