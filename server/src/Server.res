type stdin = {
  on: (string, (Buffer.t) => unit) => unit
}
type process = {
  stdin: stdin
}
@val external process: process = "process"
process.stdin.on("data", (chunk) => {
  let chunkString = chunk->Buffer.contents
  let message: Log.message = #String(chunkString)
  message->Log.write
})