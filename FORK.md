# lydiandy/monty fork

本目录是 [lydiandy/monty](https://github.com/lydiandy/monty)，从 [pydantic/monty](https://github.com/pydantic/monty) fork 而来，用来嵌入 gpui-monty（`HostObject`、宿主 `ui` / `db` 模块、应用目录的 `LoadHostModule`）。

能不动上游文件就不动。
fork 独有的行为放进新文件。
改共享文件要当预算：多一个 hunk，以后就多一次合并冲突。

不要把本文放到 `docs/` 或 `limitations/`。
那两处跟踪的是 pydantic/monty 的行为。

## 本工作区的 remote

| remote | URL | 角色 |
| --- | --- | --- |
| `origin` | `https://github.com/pydantic/monty.git` | 上游 |
| `fork` | `https://github.com/lydiandy/monty.git` | 本 fork |

这里的 `origin` 是 pydantic 仓库。
永远不要从这里 `git push origin`。
推 fork 用 `git push fork <local-branch>:main`。

如果直接 clone 的是 `lydiandy/monty`，那个 clone 的 `origin` **就是** fork。
把 pydantic 加成 `upstream`，合并 `upstream/main`，不要合 `origin/main`。

## fork 加了什么（文件）

新文件（上游没有，合并不会撞）：

- `crates/monty/src/embed.rs` — `HostVtable`、`HostObject`、construct / call / attr 分发
- `crates/monty/src/host_modules.rs` — 把应用目录里的 Python 当成真正的顶层模块

fork 仍会改的共享文件（冲突预算）：

| 文件 | 为什么要动 | 冲突频率 |
| --- | --- | --- |
| `crates/monty/src/intern.rs` | `Interns::new` 里 intern `ui` / `gpui` / `gpui_base` / `db` / `__gpui_view__`（不是 `StaticStrings`） | 低（`Interns::new` 本身） |
| `crates/monty/src/types/module.rs` | 宿主导出用的 `Module::named` / `set_attr_str` | 低 |
| `crates/monty/src/heap_data.rs` | `HeapData::HostObject`，以及 `is_callable` / `py_type` / `py_iter` 的分支 | 每加一种 heap payload |
| `crates/monty/src/types/type.rs` | 枚举末尾的 `Type::HostObject` | 每加一个 `Type` 变体 |
| `crates/monty/src/heap/mod.rs` | `Heap.host: HostVtableSlot`，挨着其它弱索引 | 每加一个 `Heap` 字段 |
| `crates/monty/src/bytecode/vm/call.rs` | 不再手写 match。`HostObject::py_call` 调 `embed::dispatch_call`，转发宏里放在 `Closure` / `FunctionDefaults` / `ExtFunction` 后面 | 每加一种可调用 heap 类型 |
| `crates/monty/src/dump_format.rs` | `StaticStrings` 和 `Type` 的 fingerprint 常量 | 上面每一处改动 |
| `crates/monty/src/bytecode/vm/mod.rs` | 没有单独 opcode。`load_module` 不是 stdlib 就进 `load_host_module` | 上游改 import 时 |
| `crates/monty/src/run.rs` | `Executor.host_modules` | 较少 |
| `crates/monty/src/lib.rs` | 再导出 embed 类型 | rustfmt / 导出列表抖动 |

## 为什么 2026-09-03 那次合并会冲突

上游和 fork 都在**往同一个封闭枚举末尾追加**（`StaticStrings`、`HeapData`、`Type`）。
Git 看到的是最后一个变体上的同一个 hunk。

那次合并（fork `3fdc3cd3` + 上游 7 个 commit，止于 `cf8246d7`）：

| 文件 | fork 一侧 | 上游一侧 | 处理 |
| --- | --- | --- | --- |
| `call.rs` | `HeapData::HostObject` → `embed::dispatch_call` | `HeapData::Partial` 绑定并调用 | 两臂都留 |
| `heap/mod.rs` | `host: HostVtableSlot` | `boundary_index`、`host_type_index` | 三个字段都留 |
| `heap_data.rs` | `HostObject` payload | `Partial` payload | 先 `Partial` 再 `HostObject`；`py_iter` 用 `py_type_name` |
| `types/type.rs` | `Type::HostObject` | `Type::Partial` | 先 `Partial` 再 `HostObject` |
| `intern.rs` | `Timespec` 后面的 gpui intern 块 | `Timespec` 后面的 `Partial` / `Func` / `Keywords` | 上游那三个在前，gpui 块跟后 |
| `dump_format.rs` | 旧 fork fingerprint | 旧上游 fingerprint | 枚举合并后再算一遍 |

当时用的规则：**上游新加的尾部变体保住官方判别值；fork 独有的变体挪到它们后面。**
`DUMP_VERSION` 现为 10（合入上游 Type 新变体后 HostObject 判别值后移）。
官方 dump 还能解码。
fork dump 如果已经按旧下标存了 `HostObject` / gpui `StaticStrings`，就不能了。

## 怎么写 fork 代码，后面合并才不那么疼

目标是少冲突，不是零冲突。
`HostObject` 作为真正的 heap 值，上游每加一种 heap 类型都会撞。
这可以接受（少见）。
把每个控件名塞进 `StaticStrings` 不行（常见）。

### 1. 能新建文件，就不要在共享文件里加 hunk

宿主分发、控件构造、应用目录加载已经是这样做的（`embed.rs`、`host_modules.rs`）。
继续这样。
`vm/mod.rs` 里 10 行钩子去调 `embed.rs`，比把 200 行宿主逻辑嵌进 VM 好合得多。

### 2. 不要再为宿主 UI 名字膨胀 `StaticStrings`

`intern.rs` 是最大的冲突源。
`StaticStrings` 的判别值要跟 dump 稳定，两边都只能往后追加。
上游也在同一个尾巴上追加 CPython 名字（`Partial`、`Func`、…）。

控件名（`Button`、`v_flex`、`Checkbox`、…）不需要静态判别值。
`Module::set_attr` 吃 `impl Into<StringId>`。
在创建模块时 intern（`vm.interns.intern("Button")`），不要加 `StaticStrings` 变体。

只给字节码或 `StandardLib` 查找真正需要的名字留一份**很短**的静态表。
产品模块（`ui` / `db` / `tui` / `reqwest`）走 `create_native_module`，不要写进 `intern.rs`，也不要写进 `StandardLib`。

### 3. fork 变体永远放最后

封闭枚举避不开时（`HeapData`、`Type`、`StandardLib`）：

- 追加到上游当前尾巴后面
- 永远不要插到中间
- 合并时重放：先收下上游新尾巴，再把 fork 变体接回去

`StandardLib::Gc` 有门控，必须保持最后（上游 dump 规则）。
产品模块不进 `StandardLib`。2026-09-16 已删 `Gpui` / `GpuiBase` / `Ui` / `Db` 墓碑，`DUMP_VERSION` 升到 9。不要再加回来。

### 4. 宿主状态能放 `VM` / `Executor`，就不要放 `Heap`

`Heap.host` 挨着 `ext_function_cache`。
上游在同一处加了 `boundary_index`。
vtable 不序列化（恢复出来的 dump 没有宿主）。

如果把 `HostVtableSlot` 挪到 `VM` 或 `Executor`（它们已经握着 `host_modules`），`heap/mod.rs` 就不用再进冲突预算。
`heap.rs` 也是安全边界；即便不考虑合并，fork hunk 越少越好。

### 5. 可调用 / 可迭代的 match 臂：钩默认分支，或只在末尾留一个具名臂

`call.rs` 冲突，是因为两边在同一个 `match` 里各加了一个可调用臂。
只留一个 `HeapData::HostObject` 臂，放在上游可调用对象（`Partial`、…）**后面**。
如果转发宏已经覆盖（`heap_read_output_py_trait_forward!`），不要手工把 `HostObject` 洒进每个 `PyTrait` 方法。

`is_callable` / `is_gc_tracked` / `py_iter` 这类 or-pattern，把 `Self::HostObject(_)` 加在组末尾，这样上游新变体是单独一行。

### 6. dump fingerprint 当合并步骤，不当设计问题

`StaticStrings` 或 `Type` 一变，`dump_format.rs` 就会对不上。
枚举解决完之后跑：

```bash
cargo test -p monty --offline serialized_components_match_dump_version -- --nocapture
```

把 `left:` 的哈希贴进那两处 `assert_eq!`。
只是往尾巴追加、还保住上游判别值时，不要升 `DUMP_VERSION`。

### 7. 不要做的事

- 不要另搞一张旁表，好让 `HostObject` 不是 `HeapData` 变体。
  call、attr、repr、GC 都需要真正的 heap 值；多出来的复杂度，比每加一种 heap 类型冲突一个 match 臂更糟。
- 不要为嵌入器功能去改 `crates/monty/src/heap.rs` 内部（分页 arena）。
- 不要为了导出控件，在 `StandardLib` 里再复制一份 CPython stdlib。
  `ui` / `db` / `tui` / `reqwest` 只走 `HostVtable::create_native_module`。
  不要再加 `StandardLib` 墓碑。

## 取舍

| 留在 fork（代价是偶发冲突） | 从共享文件挪走（代价是一次性重构） |
| --- | --- |
| `HeapData::HostObject` + `Type::HostObject` | 控件名离开 `StaticStrings` |
| `HostObject::py_call` | `Heap.host` → `VM` / `Executor` |
| `load_module` 落到 `load_host_module` | 宿主 import 正文已经在 `host_modules.rs` / `embed.rs` |
| 每次合并后更新 dump fingerprint | — |

我们接受**一种**新的 heap/type 变体带来的冲突。
不能接受上游每加 `str` 方法或 stdlib 名字，就要追加 400 行 intern。

## 合并记录

### 2026-09-03

- 把 `origin/main`（`cf8246d7`）合进 fork `main`（`3fdc3cd3`）。
- 上游 7 个 commit：`#682` ClassInstance 接线、`#796` 类型名、`#804` set repr、`#794` gather stack、`#797` builtin methods、`#787` `functools.partial`、`#802` typeshed TypedDict 内部。
- 冲突：上表那六个文件（11 个 hunk）。
- 结果：本地 `fork-main` 上的 `8b023e54 Merge origin/main into fork-main`。
- `cargo check -p monty` 和 dump fingerprint 测试通过。
- `make lint-rs` 仍会因 fork 里原有的 clippy 失败：`embed.rs` / `host_modules.rs`（`absolute_paths`、`allow` vs `expect`、…）。跟这次合并无关。

### 2026-09-03（控件名离开 `StaticStrings`）

- 从 `StaticStrings` 去掉 gpui intern 块（约 400 个变体）。
- `Interns::new` 把 `ui`、`gpui`、`gpui_base`、`db`、`__gpui_view__` intern 进动态池。
- 宿主模块属性用 `Module::set_attr_str`（已有 intern 键就用，否则 heap `str`）。
- `static_strings_fingerprint` 重新对上上游的 `0x0a4d_48bb_642d_1476`。
- 新控件由 host 登记，不要写进 `intern.rs`，不要写进 `StandardLib`。

## 命令：手工合并上游

在仓库根、干净工作区上跑。

```bash
# 1. remotes（每个 clone 做一次）
git remote -v
# origin -> github.com/pydantic/monty.git
# fork   -> github.com/lydiandy/monty.git
# 如果没有 fork：
#   git remote add fork https://github.com/lydiandy/monty.git

# 2. fetch
git fetch origin
git fetch fork

# 3. 跟踪 fork 的分支
git checkout -B fork-main fork/main
# 如果已经有 fork-main：
#   git checkout fork-main && git merge --ff-only fork/main

# 4. 合并上游
git merge origin/main
```

如果 git 说 `Already up to date`，停。
如果是 fast-forward，跳到第 7 步。

有冲突时：

```bash
# 5. 列出冲突
git diff --name-only --diff-filter=U

# 封闭枚举：先留上游新尾巴，再把 fork 独有变体接回去
# （HostObject。产品模块不进 StandardLib）。
# 不要往 StaticStrings 加工件名。
# Heap：host + boundary_index + host_type_index 都留。
# call.rs：HostObject 和 Partial（以及上游新的可调用对象）都留。

# 6. fingerprint（intern.rs / types/type.rs 解决完之后）
cargo test -p monty --offline serialized_components_match_dump_version -- --nocapture
# 失败就把 `left:` 那个 u64 写成 dump_format.rs 里的 0xHHHH_HHHH_HHHH_HHHH。

make format-rs
cargo check -p monty --offline

git add -u
git status   # 不能再有 UU 文件
git commit   # 默认 merge 说明即可
```

推到 GitHub 上的 fork（不要推到 pydantic）：

```bash
# 7. push
git push fork fork-main:main
```

GitHub 提示冲突时，**不要**点 “Sync fork” / “Discard N commits”。
那会把 fork 重置成上游，gpui-monty 的 commit 全没。
用上面的合并流程。

## 每次合并后记在这里

加一节 `### YYYY-MM-DD`，写上：

- 上游 tip SHA，以及进来了多少个 commit
- 冲突路径（或「无」）
- 哪些 dump 判别值挪过位置
- push 之后的 merge commit SHA

## 2026-09-05 — browser hostModules（slice 1）

为 monty-ui web 真 import 打通（无 MountDir）：

- proto `HostModuleSource` + `Feed.host_modules`
- WIT `host-module-source` / `feed-request.host-modules`
- `MontyRepl::feed_start_with_host_modules` + `Executor::new_repl_snippet(..., extra)`
- TS `FeedOptions.hostModules` → wasm worker transport

**不**包含原生 `ui`/`motion` HostVtable。详见 monty-ui `doc/01idea/motion/web-jco-bridge.md`。

### 这次动到的独立 crate（竖切，不只 `monty`）

| # | 包 | 大概动了什么 | 行数感 |
| --- | --- | --- | --- |
| 1 | **`monty`** | `run.rs` / `repl.rs`：REPL/执行路径接 `HostModuleSource` | 主逻辑 |
| 2 | **`monty-wasm-runtime`** | `wit/runtime.wit` + `lib.rs`：feed 带 host-modules | 小 |
| 3 | **`monty-js`** | `session` / worker transport / 类型：把 modules 传到 wasm | 中 |
| 4 | **`monty-proto`** | `monty.proto` + 生成代码 + worker：协议字段 | 中（生成文件吵） |
| 5 | **`monty-pool`** | checkout / telemetry / 测试：跟上 FeedOptions | 很小 |
| 6 | **`monty-runtime`** | 仅测试跟字段 | 极小 |

落地时约 **6 个 crate、26 个文件、+264 / −12**（另有 smoke / lockfile 一类附属）。

### 合并上游会不会变难？（结论）

**会比「只改 monty 一个文件」难一截，但还在可控范围**——前提是别再横向扩。

| | 说明 |
| --- | --- |
| **难在哪** | 这是一条**竖切**：Embed/`run` → WIT → JS → proto → pool。上游只要改 **feed / dispatch / proto** 同名表面，就容易冲突；`monty-proto` **生成文件**尤其爱撞。 |
| **还好的地方** | 总量不大；大多是**加字段 / 加参数**，不是大改算法；`run.rs` 在本文件冲突预算里本来就偏低；**没有**再动 `HeapData` / `Type` / `call.rs` 那种「枚举末尾必撞」的老痛点。 |
| **和历史比** | 2026-09-03 那种 `HostObject` vs 上游 `Partial` 的枚举尾冲突，比这次危险。这次更像「协议多一个可选列表」。 |

**一句话：** 为浏览器真 `import` 改了约 6 个包；合并会比单包 fork 麻烦，但当前体量还可维护。继续横向扩 stub / 旁路包，才会真正把合并成本顶爆。

### 以后合上游时：先看这些路径

按冲突概率优先检查（解决顺序建议）：

1. `crates/monty-proto/**`（尤其 `monty.proto` 与生成物）— 生成文件吵，先定协议再重生
2. `crates/monty-wasm-runtime/wit/runtime.wit` + 对应 `lib.rs` 映射
3. `crates/monty-js/**`（`session` / worker transport / `FeedOptions`）
4. `crates/monty/src/run.rs`、`repl.rs`（薄接线；独有逻辑尽量仍在 `host_modules.rs`）
5. `monty-pool` / `monty-runtime` — 通常只是字段跟随，冲突小

封闭枚举（`HeapData` / `Type` / `call.rs` / `intern` StaticStrings）仍按本文前面「fork 变体永远放最后」处理；**本 slice 不应新增这类 hunk**。

### 少疼规则（相对本竖切）

1. **fork 独有逻辑收在新文件**（已有 `host_modules.rs` / `embed.rs`）；共享文件只留薄接线。
2. **proto / WIT 变更单独小 commit**，方便 rebase / cherry-pick。
3. 合并时优先看上表四条路径；生成代码冲突时：**先合 `.proto` / `.wit` 语义，再重新生成**，不要手改生成文件硬拼。
4. 若上游以后**官方也加 host modules**：应**弃 fork 字段、跟官方命名与形状**，别双轨长留。
5. **不要**为了省事再横向加旁路 crate / 复制一套 feed 栈；竖切保持窄，合并成本才可控。

### 2026-09-22 — 跟随 `v1.0.0-beta.2`

- 上游 tip `64662cc5`（`v1.0.0-beta.2`），自 merge-base `4f0fdd4d` 起 35 个 commit。
- 冲突：`tracing.rs`、`monty.proto`、生成的 `monty.v1.rs`、`worker.rs`、`compiler.rs`、`op.rs`、`call.rs`、`vm/mod.rs`、`dump_format.rs`、`heap/mod.rs`、`heap_data.rs`、`intern.rs`、`lib.rs`、`repl.rs`、`run.rs`、`module.rs`。
- `LoadModule` 的操作数已是模块名，不再保留 fork 的 `LoadHostModule` opcode（那个槽位现在是上游的 `LoadName`）。未知模块在 `load_module` 里落到 `load_host_module`。
- `HostObject` 仍在 `HeapData` / `Type` 尾巴。调用走 `PyTrait::py_call`，不再在 `call.rs` 里留 match 臂。
- `DUMP_VERSION` 跟上游，仍是 12。`Type` fingerprint 因尾巴上的 `HostObject` 与上游不同，是 `0xb92a_0953_8a54_fe3b`。opcode fingerprint 与上游相同。
- proto `Feed.host_modules` 是字段 6（`cwd` 是 5）。生成代码按 schema 重生，没有手拼。
- 本 clone 的 `origin` 是 fork，合的是 `upstream/main`。
- merge commit `c03faf05`。
