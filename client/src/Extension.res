@val external __dirname: string = "__dirname"
type serverOptions = {
  run: {"module": string, "transport": int},
  debug: {"module": string, "transport": int},
}
type transportKind = {stdio: int, ipc: int}

type documentSelector = {
  scheme: string,
  language: string,
}
type syncOptions = {change: bool}
type synchronize = {fileEvents: syncOptions}
type languageClientOptions = {
  documentSelector: array<documentSelector>,
  synchronize: synchronize,
}
type client = {
  start: unit => unit,
  stop: unit => unit,
}

@module("vscode-languageclient/node") @new
external createLanguageClient: (
  string,
  string,
  serverOptions,
  languageClientOptions,
) => client = "LanguageClient"
type workspace = {createFileSystemWatcher: string => bool}
@module("vscode") external workspace: workspace = "workspace"
@module("vscode-languageclient/node") external serverOptions: serverOptions = "ServerOptions"
@module("vscode-languageclient/node") external transportKind: transportKind = "TransportKind"
@module("vscode-languageclient/node")
external languageClientOptions: languageClientOptions = "LanguageClientOptions"
module Path = {
  @module("path") @variadic external join: array<string> => string = "join"
}
type extensionContext = {
  asAbsolutePath: string => string,
}
@module("vscode") external extensionContext: extensionContext = "ExtensionContext"

let client: ref<option<client>> = ref(None)
let activate = (context: extensionContext) => {
  let serverModule = Path.join(["lib", "js", "server", "src", "Server.js"]) |> context.asAbsolutePath
  let serverOptions = {
    run: {"module": serverModule, "transport": transportKind.stdio},
    debug: {"module": serverModule, "transport": transportKind.stdio},
  }

  let clientOptions = {
    // documentSelector: [
    //   {
    //     scheme: "file",
    //     language: "cfml"
    //   },
    //   {
    //     scheme: "file",
    //     language: "cfc"
    //   }
    // ],
    documentSelector: [{scheme: "file", language: "*"}],
    synchronize: {
      fileEvents: {
        // Notify the server about file changes to '.clientrc files contain in the workspace
        change: workspace.createFileSystemWatcher("**/.clientrc"),
      },
    },
  }

  client := Some(createLanguageClient("cfml", "CFML Language Server", serverOptions, clientOptions))
  switch (client.contents) {
  | Some(client) => {
    Console.log(client)
    client.start()
  }
  | None => ()
  }
}

let deactivate = () => {
  switch (client.contents) {
  | Some(client) => client.stop()
  | None => ()
  }
}
