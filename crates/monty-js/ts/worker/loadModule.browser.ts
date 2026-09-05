/// <reference lib="dom" />
// 拉取并编译 Monty component 的各 core 模块。

import { COMPONENT_MODULE_NAMES } from './componentModules.js'
import type { ComponentModules } from './host.js'

/** 加载实例化所需的全部 core 模块。 */
export async function loadModule(): Promise<ComponentModules> {
  // URL 字面量保持内联，便于 Vite/webpack 把 component 资产打进包。
  const modules = await Promise.all([
    WebAssembly.compileStreaming(fetch(new URL('./component/monty.component.core.wasm', import.meta.url))),
    WebAssembly.compileStreaming(fetch(new URL('./component/monty.component.core2.wasm', import.meta.url))),
    WebAssembly.compileStreaming(fetch(new URL('./component/monty.component.core3.wasm', import.meta.url))),
    WebAssembly.compileStreaming(fetch(new URL('./component/monty.component.core4.wasm', import.meta.url))),
    WebAssembly.compileStreaming(fetch(new URL('./component/monty.component.core5.wasm', import.meta.url))),
  ])
  return Object.fromEntries(COMPONENT_MODULE_NAMES.map((name, index) => [name, modules[index]]))
}
