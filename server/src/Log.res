
type file = {
    write: string => unit
}
@module("fs") external createWriteStream: string => file = "createWriteStream"

let log = createWriteStream("/tmp/lsp.log")

type message = [
    #Json(Js.Json.t)
    | #String(string)
]

let write = (message) => {
    let messageString = switch message {
    | #Json(json) => Js.Json.stringify(json)
    | #String(string) => string
    }
    log.write(messageString ++ "\n")
}