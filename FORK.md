# lydiandy/monty fork

This tree is [lydiandy/monty](https://github.com/lydiandy/monty), a fork of
[pydantic/monty](https://github.com/pydantic/monty) that embeds gpui-monty
(`HostObject`, host `ui` / `db` modules, app-dir `LoadHostModule`).

Upstream-only files stay untouched when we can.
Fork-only behaviour lives in new files.
Shared files are a budget: every extra hunk there is a future merge conflict.

Do not put this document in `docs/` or `limitations/`.
Those track pydantic/monty behaviour.

## Remotes in this checkout

| remote | URL | role |
| --- | --- | --- |
| `origin` | `https://github.com/pydantic/monty.git` | upstream |
| `fork` | `https://github.com/lydiandy/monty.git` | this fork |

In this checkout `origin` is the pydantic repo.
Never `git push origin` from here.
Push the fork with `git push fork <local-branch>:main`.

If you cloned `lydiandy/monty` directly, that clone's `origin` *is* the fork.
Add pydantic as `upstream` and merge `upstream/main` instead of `origin/main`.

## What the fork adds (files)

New files (upstream does not have them; they do not conflict on merge):

- `crates/monty/src/embed.rs` — `HostVtable`, `HostObject`, construct/call/attr dispatch
- `crates/monty/src/host_modules.rs` — app-dir Python as real top-level modules
- `crates/monty/src/modules/ui.rs`
- `crates/monty/src/modules/gpui.rs`
- `crates/monty/src/modules/gpui_base.rs`
- `crates/monty/src/modules/db.rs`

Shared files the fork still patches (conflict budget):

| file | why the fork touches it | conflict frequency |
| --- | --- | --- |
| `crates/monty/src/intern.rs` | `Interns::new` intern of `ui` / `gpui` / `gpui_base` / `db` / `__gpui_view__` (not `StaticStrings`) | rare (`Interns::new` itself) |
| `crates/monty/src/types/module.rs` | `Module::named` / `set_attr_str` for host exports | rare |
| `crates/monty/src/heap_data.rs` | `HeapData::HostObject` plus `is_callable` / `py_type` / `py_iter` arms | every new heap payload |
| `crates/monty/src/types/type.rs` | `Type::HostObject` at the enum tail | every new `Type` variant |
| `crates/monty/src/heap/mod.rs` | `Heap.host: HostVtableSlot` next to the other weak indexes | every new `Heap` field |
| `crates/monty/src/bytecode/vm/call.rs` | `HostObject` match arm next to other callables | every new callable heap type |
| `crates/monty/src/dump_format.rs` | fingerprint constants for `StaticStrings` and `Type` | every change above |
| `crates/monty/src/modules/mod.rs` | `StandardLib::{Gpui,GpuiBase,Ui,Db}` tombstones before `Gc` | every new stdlib module |
| `crates/monty/src/bytecode/vm/mod.rs` | `LoadHostModule` / `host_modules` on `VM` | whenever upstream edits import |
| `crates/monty/src/run.rs` | `Executor.host_modules` | less often |
| `crates/monty/src/lib.rs` | re-export embed types | rustfmt / export list churn |

## Why the 2026-09-03 merge conflicted

Upstream and the fork both **append** to the same closed enums
(`StaticStrings`, `HeapData`, `Type`).
Git sees one hunk at the last variant.

That merge (fork `3fdc3cd3` + 7 upstream commits ending at `cf8246d7`):

| file | fork side | upstream side | resolution |
| --- | --- | --- | --- |
| `call.rs` | `HeapData::HostObject` → `embed::dispatch_call` | `HeapData::Partial` bind-and-call | keep both arms |
| `heap/mod.rs` | `host: HostVtableSlot` | `boundary_index`, `host_type_index` | keep all three fields |
| `heap_data.rs` | `HostObject` payload | `Partial` payload | `Partial` then `HostObject`; `py_iter` uses `py_type_name` |
| `types/type.rs` | `Type::HostObject` | `Type::Partial` | `Partial` then `HostObject` |
| `intern.rs` | gpui intern block after `Timespec` | `Partial` / `Func` / `Keywords` after `Timespec` | upstream three first, gpui block after |
| `dump_format.rs` | old fork fingerprints | old upstream fingerprints | recompute after the enum merge |

Rule used: **upstream's new tail variants keep their official discriminants;
fork-only variants move after them.**
`DUMP_VERSION` stayed 8.
Official dumps still decode.
A fork dump that already stored `HostObject` / gpui `StaticStrings` under the
old indexes does not.

## How to write fork code so later merges hurt less

Goal is fewer conflicts, not zero.
`HostObject` as a real heap value will keep colliding when upstream adds a
heap type.
That is acceptable (rare).
Putting every widget name in `StaticStrings` is not (common).

### 1. Prefer a new file over a hunk in a shared file

Host dispatch, widget constructors, and app-dir loading already follow this
(`embed.rs`, `modules/ui.rs`, `host_modules.rs`).
Keep doing that.
A 10-line hook in `vm/mod.rs` that calls into `embed.rs` is cheaper to merge
than 200 lines of host logic inlined in the VM.

### 2. Stop growing `StaticStrings` for host UI names

`intern.rs` is the largest conflict.
`StaticStrings` discriminants are dump-stable, so both sides can only append.
Upstream appends CPython names (`Partial`, `Func`, …) at the same tail.

Widget names (`Button`, `v_flex`, `Checkbox`, …) do not need a static
discriminant.
`Module::set_attr` takes `impl Into<StringId>`.
Intern them at module-create time (`vm.interns.intern("Button")`) instead of
adding a `StaticStrings` variant.

Keep a **short** static list only for names that bytecode or `StandardLib`
lookups actually require (module names such as `ui` / `gpui` if dumps still
need those ids).
New widgets go in `modules/ui.rs`, not `intern.rs`.

### 3. Fork variants always last

When a closed enum is unavoidable (`HeapData`, `Type`, `StandardLib`):

- append after whatever upstream currently has at the tail
- never insert in the middle
- on merge, replay: take upstream's new tail, then re-append the fork variants

`StandardLib::Gc` is gated and must remain last (upstream dump rule).
Fork tombstones `Gpui` / `GpuiBase` / `Ui` / `Db` sit just before `Gc`.
A new upstream module will land in that same slot; move the tombstones after
it, still before `Gc`.

### 4. Do not put host state on `Heap` if it can live on `VM` / `Executor`

`Heap.host` sat next to `ext_function_cache`.
Upstream added `boundary_index` in the same place.
The vtable is not serialized (a restored dump has no host).

If we move `HostVtableSlot` onto `VM` or `Executor` (which already hold
`host_modules`), `heap/mod.rs` drops out of the conflict budget.
`heap.rs` is also a security boundary; fewer fork hunks there is better even
aside from merges.

### 5. Callable / iterable match arms: hook the default, or keep one named arm at the end

`call.rs` conflicted because both sides added a callable arm in the same
`match`.
Keep a single `HeapData::HostObject` arm **after** upstream callables
(`Partial`, …).
Do not sprinkle `HostObject` into every `PyTrait` method by hand if the
forwarding macro already covers it (`heap_read_output_py_trait_forward!`).

For `is_callable` / `is_gc_tracked` / `py_iter` or-patterns, add
`Self::HostObject(_)` at the end of the group so a new upstream variant is a
separate line.

### 6. Treat dump fingerprints as a merge step, not a design problem

`dump_format.rs` will disagree every time `StaticStrings` or `Type` changes.
After resolving enums, run:

```bash
cargo test -p monty --offline serialized_components_match_dump_version -- --nocapture
```

Paste the `left:` hashes into the two `assert_eq!`s.
Do not bump `DUMP_VERSION` for a tail-append that keeps upstream discriminants.

### 7. What not to do

- Do not invent a side table so `HostObject` is not a `HeapData` variant.
  Call, attr, repr, and GC all need a real heap value; the extra complexity
  is worse than one match-arm conflict per new heap type.
- Do not edit `crates/monty/src/heap.rs` internals (the paged arena) for
  embedder features.
- Do not duplicate CPython stdlib in `StandardLib` just to export widgets.
  `ui` / `db` already load through `HostVtable::create_native_module`.
  The remaining `StandardLib::{Gpui,Ui,Db}` variants are dump tombstones;
  do not add more.

## Balance

| keep on the fork (cost is rare conflicts) | move off shared files (cost is a one-time refactor) |
| --- | --- |
| `HeapData::HostObject` + `Type::HostObject` | widget names out of `StaticStrings` |
| one `call.rs` arm | `Heap.host` → `VM` / `Executor` |
| `LoadHostModule` hook in `vm/mod.rs` | host import body already in `host_modules.rs` / `embed.rs` |
| dump fingerprint updates after each merge | — |

We accept conflicts on **one** new heap/type variant.
We do not accept a 400-line intern append every time upstream adds `str` methods
or a stdlib name.

## Merge log

### 2026-09-03

- Merged `origin/main` (`cf8246d7`) into fork `main` (`3fdc3cd3`).
- Upstream commits (7): `#682` ClassInstance wire, `#796` type names, `#804` set repr,
  `#794` gather stack, `#797` builtin methods, `#787` `functools.partial`,
  `#802` typeshed TypedDict internals.
- Conflicts: the six files in the table above (11 hunks).
- Result: `8b023e54 Merge origin/main into fork-main` on local `fork-main`.
- `cargo check -p monty` and the dump fingerprint test passed.
- `make lint-rs` still fails on pre-existing fork clippy in `embed.rs` /
  `host_modules.rs` / `modules/ui.rs` / `modules/gpui.rs` (`absolute_paths`,
  `allow` vs `expect`, …). Not part of the merge.

### 2026-09-03 (widget names off `StaticStrings`)

- Removed the gpui intern block from `StaticStrings` (~400 variants).
- `Interns::new` intern `ui`, `gpui`, `gpui_base`, `db`, `__gpui_view__` into the
  dynamic pool.
- Host module attrs use `Module::set_attr_str` (interned key if present, else
  heap `str`).
- `static_strings_fingerprint` matches upstream `0x0a4d_48bb_642d_1476` again.
- New widgets go in `modules/ui.rs`, not `intern.rs`.

## Commands: merge upstream by hand

Run from the repo root, on a clean tree.

```bash
# 1. remotes (once per clone)
git remote -v
# origin -> github.com/pydantic/monty.git
# fork   -> github.com/lydiandy/monty.git
# if fork is missing:
#   git remote add fork https://github.com/lydiandy/monty.git

# 2. fetch
git fetch origin
git fetch fork

# 3. branch that tracks the fork
git checkout -B fork-main fork/main
# or, if you already have fork-main:
#   git checkout fork-main && git merge --ff-only fork/main

# 4. merge upstream
git merge origin/main
```

If git reports `Already up to date`, stop.
If it fast-forwards, skip to step 7.

On conflicts:

```bash
# 5. list conflicts
git diff --name-only --diff-filter=U

# Closed enums: keep upstream's new tail variants, then re-append fork-only
# variants (HostObject, StandardLib tombstones before Gc).
# Do not add widget names to StaticStrings.
# Heap: keep host + boundary_index + host_type_index.
# call.rs: keep HostObject and Partial (and any new upstream callable).

# 6. fingerprints (after intern.rs / types/type.rs are resolved)
cargo test -p monty --offline serialized_components_match_dump_version -- --nocapture
# On failure, put the `left:` u64 into dump_format.rs as 0xHHHH_HHHH_HHHH_HHHH.

make format-rs
cargo check -p monty --offline

git add -u
git status   # no UU files
git commit   # default merge message is fine
```

Publish to the GitHub fork (not to pydantic):

```bash
# 7. push
git push fork fork-main:main
```

Do **not** click GitHub "Sync fork" / "Discard N commits" when it reports
conflicts.
That resets the fork to upstream and drops the gpui-monty commits.
Use the merge above instead.

## After each merge, record it here

Add a `### YYYY-MM-DD` section with:

- upstream tip SHA and how many commits came in
- conflicted paths (or "none")
- anything whose dump discriminant moved
- the merge commit SHA after push
