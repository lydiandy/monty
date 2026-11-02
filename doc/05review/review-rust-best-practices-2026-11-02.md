# Rust 最佳实践评审（monty-fork，2026-11-02）

依据：`rust-best-practices` skill（13 节）。硬约束：`FORK.md`（优先新文件、封闭枚举的 fork 变体放最后、不往 `StaticStrings` 加工件名、不把 gpui / ratatui / turso 加进来）。

## 范围

对照 `upstream/main`（`ebdccb09`）到 fork `HEAD`（`415e70f7`）的 diff，只看 fork 自己的改动。上游整文件复制的 `crates/monty-*` 不评。

主要面：

- 新文件：`crates/monty/src/embed.rs`、`crates/monty/src/host_modules.rs`、`crates/monty/src/bytecode/vm/host_await.rs`
- 共享文件上的 fork hunk：`bytecode/vm/mod.rs`、`async_exec.rs`、`heap/mod.rs`、`heap_data.rs`、`intern.rs`、`types/module.rs`、`types/type.rs`、`run.rs`、`repl.rs`、`lib.rs`、`dump_format.rs`、`modules/copy.rs`、`name_map.rs`、`object_bridge.rs`、`source_map.rs`
- 宿主模块竖切：`monty-proto` worker、`monty-pool` checkout、`monty-js` session / napi、`monty-wasm-runtime`

查过、没有问题的：

- §8：fork 没有新的 `unsafe` / `transmute` / 裸指针。`HostObject` 是普通 heap 载荷，分发走安全的 `HostVtable`
- §4 / `FORK.md`：`HostObject` 仍是 `HeapData` / `Type` 最后一个变体。`ui` / `gpui` / `gpui_base` / `db` / `__gpui_view__` 在 `Interns::new` 的动态池（`intern.rs:2904`），没有进 `StaticStrings`。产品模块不进 `StandardLib`
- §11：`serialized_components_match_dump_version` 通过
- §7：`HostModuleRegistry` 的 `RefCell` 借用在返回前结束，没有把 `Ref` 守卫漏出函数

## 总量

17 条。高 4、中 5、低 8。修了 9 条（高 3、中 4、低 2）。未修 8 条。

## 已修

### 1. 高，§5。`drive` 失败会丢掉模块全局

`crates/monty/src/embed.rs:731`

`Embed::drive` 先 `mem::take` 走 `globals`，错误分支直接丢弃 VM。`VM` 的 `Drop` 会 `drop_with` 这些全局，会话里的名字表变成空的，下一次 `call_global` 按下标取就会越界。

建议：错误路径也 `take_globals`，跟 `with_script_call` 一样把名字表放回去。

状态：已修。回归 `embed::tests::failed_drive_keeps_module_globals`。

### 2. 高，§5。恢复中的 host await 失败同样丢掉全局

`crates/monty/src/embed.rs:804`

`run_parked` 从快照恢复 VM，快照已经拿走全局。`resolve_future` / `fail_future` 若脚本没接住异常，错误分支不把全局放回 `Embed`。停着的任务被 `Drop` 清掉可以，模块名字表不行。

建议：错误时 `take_globals`，会话回到空闲，后续调用还能看见原来的全局。

状态：已修。回归 `uncaught_fail_future_keeps_module_globals`（`crates/monty/tests/host_await.rs`）。

### 3. 高，§5。宿主模块导入失败后仍留在缓存里

`crates/monty/src/bytecode/vm/mod.rs:2153`（以及 `2166`、`2217`）、`crates/monty/src/host_modules.rs:96`

`load_host_module` 在 `begin_load` 之后，无论是重入限额、压帧失败还是模块体抛错，都不 `finish_load`，也不把缓存项拿掉。`cached` 不看 `loaded`，下一次 `import` 直接交回半成品模块，还漏掉 `allocate` 那次引用。

建议：失败时 `abort_load`，只摘掉还没 `finish` 的项，并由调用方 `dec_ref`。已经加载完的不动。

状态：已修。回归 `embed::tests::failed_host_module_import_is_not_cached`。

### 4. 中，§5。宿主模块 `set_attr` 吞掉插入错误

`crates/monty/src/types/module.rs:130`

fork 加的 `HeapReadMut<Module>::set_attr_key` 对 `Dict::set` 的 `Err` 什么都不做。上游 `Module::set_attr`（同文件 `102` 行）写明模块属性键是已 intern 的字符串，哈希不会失败，并且跳过增长预检。fork 这条既做了预检，又把失败吞掉，内存压力下赋值会静默丢失。

建议：跟上游一样 `set_without_growth_check`，失败用 `expect` 说明不变量，不要吞。

状态：已修。签名仍是 `()`，stdlib 那批 `module.set_attr` 不用改。

### 5. 中，§5。坏的 `KIND_BOXED_MODULE_FN` 载荷会 panic

`crates/monty/src/embed.rs:1612`

`unpack_module_function` 用 `expect` 解 postcard。宿主可以 `alloc` 一个 `kind == KIND_BOXED_MODULE_FN` 且 `data` 任意的对象，调用它会在解释器里 panic。这是宿主输入，不是内部不变量。

建议：长度超过 7 或解码失败返回 `Err`，`dispatch_call` 转成 `RuntimeError`。自己 `pack` 出来的字节仍然 `expect` / `assert`，那是我们的编码不变量，断言带上长度。

状态：已修。回归 `embed::tests::corrupt_boxed_module_function_is_an_error`。

### 6. 中，§5。脚本调用只在成功路径释放可调用对象

`crates/monty/src/embed.rs:508`（`call_global`）、`593`（`call_method`）、`627`（`call_callable`）、`1490`（`call_pending`）

`inc_ref` / `clone_with_heap` 之后用 `?`，`drop_with` 写在 `?` 后面。求值或查方法失败时，多出来的引用不会还。`call_method` 在 `lookup_method` 失败时，已经 prepend 进参数的 `self` 也会被普通 `Drop` 丢掉（`memory-model-checks` 下会 panic，默认构建是静默泄漏）。

建议：先拿到 `Result`，再 `drop_with`，然后传播错误。`call_callable` 只保留一个 `Value::Ref` 拥有那次 `inc_ref`，`ModuleFunction` 作为调用目标另放，避免两份 `Value::Ref` 指同一个 id。pending 调用同样在两条路径上都释放。

状态：已修。引用计数本身没有开 `memory-model-checks`（该特性极慢，不进这次命令）。行为回归覆盖了失败后会话仍可用。

### 7. 中，§5。参数转成宿主值时，中途失败会漏引用

`crates/monty/src/embed.rs:1434`

`args_to_host` 用 `?`。`value_to_host` 失败时，已经放进 `Vec<HostValue>` 的 `Heap` 没有 `Drop` 去 `dec_ref`，迭代器里剩下的 `Value` 也没有 `drop_with`。关键字名不是字符串时，对应的值同样被普通 `Drop` 丢掉。

建议：失败时 `release_host_values` 还掉已转出的 heap id，并用 `DropWithContext` 清掉剩余位置参数和关键字。

状态：已修。这条是防御路径（正常 Python 关键字名是字符串），没有单独夹具。

### 8. 低，§5。长度检查之后的 `unwrap`

`crates/monty/src/embed.rs:1636`

`host_values_to_args` 在 `len` 已判断后再 `pop().unwrap()`。不是可失败输入，但是库代码里的裸 `unwrap`。

建议：改成带原因的 `expect`。

状态：已修。

### 9. 低，§2。fork 留下的多余 `mut`

`crates/monty/src/bytecode/vm/async_exec.rs:637`

fork 把 `init_task_from_coroutine` 改成 `pub(super)` 时加了用不上的 `mut coro`。同一次编译里 `embed.rs` 两处 `Dict` 绑定也是多余 `mut`（`HeapReadMut::set` 吃 `&self`）。

建议：去掉 `mut`。

状态：已修。

## 未修

### 10. 高，§4。`FeedOptions.hostModules` 到了边界被丢掉

`crates/monty-pool/src/checkout.rs:928`
`crates/monty-js/src/pool.rs:219`
`crates/monty-wasm-runtime/src/lib.rs:324`
`crates/monty-js/ts/session.ts:249`
`crates/monty-wasm-runtime/wit/runtime.wit` 的 `feed-request`（约 `313` 行）没有宿主模块字段

worker 已经读 proto 的 `Feed.host_modules` 并交给 `feed_start_checked_with_host_modules`。往上三处生产者都写 `Default::default()`。napi 的 `NativeFeedOptions` 只有 `cwd` / `skip_type_check`。TypeScript 把 `hostModules` 当成 `feed` 的第 6 个参数传进去，napi 和 wasm transport 都不读。`smoke-hostmodules.mjs` 描述的浏览器 / Node 真 `import` 因此到不了解释器。进程内 `Embed::run_source_with_modules` 是通的，不受这条影响。

建议：`NativeFeedOptions` 增加模块列表，`Checkout::feed_with_cwd` 写进 proto。wasm 要先给 WIT 的 `feed-request` 加字段，再 `make build-wasm` 重生绑定，不要手改生成物。

状态：未修。要动 napi、pool、python 调用点，以及 WIT 生成物，超出这次能安全落地的范围。worker 侧不用再改。

### 11. 中，§7。宿主 `RefCell` 已借出时，可迭代判断变成假

`crates/monty/src/embed.rs:1692`（`py_is_iterator` 在 `1697`，`py_iter` / `py_next` 同样）

`try_borrow()` 失败时返回 `false`，于是重入期间一个本可迭代的宿主对象会被报成 `TypeError: not iterable`。`take_pending_call` 就是为了避开「宿主回调还握着 `borrow_mut` 时再进 VM」。这条把重入藏成错误的类型判断。

建议：借不到时不要假装不可迭代。要么把 `is_iterable` 改成只看 `HostObject`（不进 vtable），要么在借失败时返回运行时错误。前者要确认 monty-ui 的判断只依赖 `kind`。

状态：未修。改判定会碰到宿主重入契约，需要 monty-ui 对一下，不在这次单独改。

### 12. 低，§12。`dict_key_at` 是线性的

`crates/monty/src/embed.rs:1306`

上游删了 `Dict::key_at` 之后，fork 用 `iter().nth`。按插入顺序走完一个 dict 是 O(n²)。`FORK.md` 记过这个替换。导入和构造不是热路径。

建议：如果宿主会按索引扫大 dict，再在 `Dict` 上恢复按插入序的 O(1) 取键，不要在 `embed.rs` 自己做第二份索引。

状态：未修。现在没有 O(1) API，新加索引会进共享 dict，合并成本不值。

### 13. 低，§3 / §4。公开边界是 `Rc<RefCell<dyn HostVtable>>` 和 `Result<_, String>`

`crates/monty/src/embed.rs` 的 `HostVtable` / `Embed::run_source`

skill 要求公开签名不要出现 `Rc` / `RefCell`，库的错误类型不要用 `String`。这里的 `RefCell` 是宿主回调重入所必需的，`String` 会被收成 `RuntimeError` 文案。换成自有错误类型会改 monty-ui 每一个 `HostVtable` 实现。

建议：保持。若以后改错误类型，先出对照表再动。

状态：未修。API 形状按 `FORK.md` 和 monty-ui 的调用约定留着。

### 14. 低，§5。借来的值把不认识的立即数变成 `None`

`crates/monty/src/embed.rs:864`、`1430`

拥有的 `value_to_host` 对不支持的值返回 `TypeError`。`borrowed_to_host` / `borrowed_value_to_host` 的 `_` 分支返回 `HostValue::None`。`dict_get` / `list_get` 走的是后者，复杂立即数会被看成缺失。

建议：两条路径用同一个匹配。改之前要确认 monty-ui 没有依赖「读到 None」。

状态：未修。行为可能已被宿主当成契约，这次不改语义。

### 15. 低，§9。`monty` 包清单相对上游少了元数据、换了 `getrandom`

`crates/monty/Cargo.toml:1`（去掉 `readme` / `homepage`）、`49`（`getrandom` 0.3 → 0.4，去掉 `itertools`）

这是 fork 相对上游的打包漂移，用来躲开 cargo 清单 lint，并让 wasm 上的 `getrandom` 能编。把 `readme` / `homepage` 加回去会让 `heap_reader_compile_fail` 的 stderr 更吵，解决不了那 24 条已知红（见测试）。

建议：合并上游时单独看这几行，不要顺手把元数据改回上游而不重看 compile-fail 过滤。

状态：未修。不是行为 bug，回退会扩大 diff。

### 16. 低，§11。多字节诊断的测试写在模块里面

`crates/monty/src/source_map.rs:187`

fork 修了诊断偏移落在多字节字符中间时的 panic，测试放在 `source_map.rs` 的 `#[cfg(test)]`。本仓 `AGENTS.md` 要求测试放 `tests/`。skill 允许模块内单测。函数是私有的，搬出去要加测试可见性。

建议：留在原地。它断言的是「不 panic」，已经比没有强。

状态：未修。搬位置没有行为收益。

### 17. 低，§4。控件 `KIND_*` 常量堆在解释器里

`crates/monty/src/embed.rs:28` 起

一百多个控件 kind 是宿主协议，不是 `StaticStrings`。`FORK.md` 禁止的是把控件名塞进 `StaticStrings` / `StandardLib`，这条没有违反。常量已经在 fork 新文件里，并由 `lib.rs` 再导出。挪到 monty-ui 会改公开常量路径。

建议：保持。新 kind 只追加，不改号。

状态：未修。符合 `FORK.md`，不是要拆的第二套实现。

## 测试

```bash
cargo.rustup +nightly-2026-10-25 test -p monty -- --test-threads=1
```

`cargo` 默认在第一个失败的测试目标处停下。停在 `heap_reader_compile_fail`：24 失败 / 0 通过。失败原因与这次改动无关，`FORK.md` 2026-09-25 已记录：`normalize_stderr` 只滤 `warning:` 开头的行，新 cargo 的清单 lint 续行（`unused_workspace_dependencies`、`redundant_homepage` 等）进了 insta 快照。快照差集就是这些清单警告，外加一处 rustc 给闭包返回类型加了反引号。

该目标之前的 lib 与集成测试均通过（含 `embed` 14 条、`dump_format` 指纹）。之后的目标用同一条 `cargo.rustup` 补跑，全部通过，其中包括 `host_await` 4、`host_import` 18、`repl` 104、doctest 6 通过 / 2 ignored。

格式化：`make format-rs`。

未提交。
