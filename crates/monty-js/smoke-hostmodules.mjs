import { Monty } from './dist/worker/index.node.js'

const helloSource = [
  '# probe',
  'def greet(name="world"):',
  '    return f"hello,{name}"',
  '',
].join('\n')

const pool = await Monty.create({ minProcesses: 1, maxProcesses: 1 })
try {
  const session = await pool.checkout()
  try {
    const result = await session.feedRun('import hello\nhello.greet("monty")', {
      skipTypeCheck: true,
      hostModules: [{ name: 'hello', filename: 'hello.py', source: helloSource }],
    })
    console.log('RESULT', JSON.stringify(result))
    if (result !== 'hello,monty') {
      throw new Error('unexpected: ' + JSON.stringify(result))
    }
    console.log('HOSTMODULES_OK')
  } finally {
    await session.close()
  }
} finally {
  await pool.close()
}
