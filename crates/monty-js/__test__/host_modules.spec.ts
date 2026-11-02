// Host-supplied Python sources (`FeedOptions.hostModules`) on whichever backend
// this vitest config selected. The wasm run is the one that used to reject the list.
import { test } from 'vitest'

import { t } from './assertions.js'
import { setupPool } from './helpers.js'

const { run } = setupPool()

const helloSource = ['def greet(name="world"):', '    return f"hello,{name}"', ''].join('\n')

test('import hello calls hello.greet', async () => {
  t.is(
    await run('import hello\nhello.greet("monty")', {
      skipTypeCheck: true,
      hostModules: [{ name: 'hello', filename: 'hello.py', source: helloSource }],
    }),
    'hello,monty',
  )
})
