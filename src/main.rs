mod server;

use std::error::Error;

use lsp_server::{Connection, ExtractError, Message, Request, RequestId};
use lsp_types::{
    CompletionOptions, InitializeParams, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind,
};
fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    eprintln!("Starting ColdFusion Language Server...");

    let (connection, io_threads) = Connection::stdio();

    let server_capabilties = serde_json::to_value(&ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        completion_provider: Some(CompletionOptions {
            resolve_provider: Some(true),
            trigger_characters: Some(vec![".".to_string()]),
            all_commit_characters: None,
            completion_item: None,
            work_done_progress_options: Default::default(),
        }),
        ..Default::default()
    })
    .unwrap();

    let initialization_params = match connection.initialize(server_capabilties) {
        Ok(params) => params,
        Err(err) => {
            if err.channel_is_disconnected() {
                io_threads.join().unwrap();
            }
            return Err(err.into());
        }
    };

    run(connection, initialization_params)?;
    io_threads.join()?;
    eprintln!("ColdFusion Language Server has stopped.");
    Ok(())
}

fn run(
    connection: Connection,
    params: serde_json::Value,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    eprintln!("Initialized with params: {:?}", _params);
    for msg in &connection.receiver {
        eprintln!("Received message: {:?}", msg);
        match msg {
            Message::Request(req) => server::handle_request(req, &connection)?,
            Message::Response(resp) => server::handle_response(resp)?,
            Message::Notification(not) => server::handle_notification(not)?,
        }
    }
    Ok(())
}

fn cast<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}
