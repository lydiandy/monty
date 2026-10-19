# Host await（monty-ui fork 扩展）

2026-10-19

这个 fork 增加了一条宿主 await 路径，用来等待由宿主事件稍后 resolve 的值（对话框、HTTP、DB 等）

- `HostCtx::host_await()` 返回 `(call_id, awaitable)`，宿主从调用返回这个 awaitable
- `Embed::drive()` / `resolve_future()` / `fail_future()` 负责停车与恢复；`is_parked()` 查询状态
- 这是 `ExternalFuture`，不是新的 `__await__` 协议；用户自定义类仍然不能 await
- 停车期间可以运行同步 Python、timer、文本绑定；`with_resumed_vm` 恢复后重新停车
- `spawn` 只接受 coroutine；直接 spawn `ExternalFuture` / `GatherFuture` 会明确报错
- 其余 asyncio 限制（`async with` / `async for`、`create_task`、`Queue`、`sleep` 语义）不变
- 这是 fork 扩展，pydantic/monty 上游没有

实现位置：`crates/monty/src/embed.rs`、`crates/monty/src/bytecode/vm/host_await.rs`、
`crates/monty/src/bytecode/vm/mod.rs`、`crates/monty/src/bytecode/mod.rs`

其余 `async def` / `await` 与 asyncio 模块的限制见 [asyncio.md](asyncio.md)
