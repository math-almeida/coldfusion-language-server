use std::error::Error;

use lsp_server::{Connection, ExtractError, Message, Notification, Request, RequestId, Response};
use lsp_types::{
    request::Completion, CompletionItem, CompletionItemKind, CompletionOptions, CompletionParams, InitializeParams, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind
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

    main_loop(connection, initialization_params)?;
    io_threads.join()?;
    eprintln!("ColdFusion Language Server has stopped.");
    Ok(())
}

fn main_loop(
    connection: Connection,
    params: serde_json::Value,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    eprintln!("Initialized with params: {:?}", _params);
    for msg in &connection.receiver {
        eprintln!("Received message: {:?}", msg);
        match msg {
            Message::Request(req) => handle_request(req, &connection)?,
            Message::Response(resp) => handle_response(resp)?,
            Message::Notification(not) => handle_notification(not)?,
        }
    }
    Ok(())
}

fn handle_request(
    req: Request,
    connection: &Connection,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    if connection.handle_shutdown(&req)? {
        return Ok(());
    }
    match cast::<Completion>(req) {
        Ok((id, params)) => {
            let result = handle_completion(params);
            let resp = Response::new_ok(id, result);
            connection.sender.send(Message::Response(resp))?;
            Ok(())
        },
        Err(err @ ExtractError::JsonError { .. }) => panic!("JSON error: {}", err),
        Err(ExtractError::MethodMismatch(req)) => {
            println!("Method mismatch: {:?}", req);
            Ok(())
        }
    }
}

fn handle_response(resp: Response) -> Result<(), Box<dyn Error + Sync + Send>> {
    eprintln!("Received response: {:?}", resp);
    Ok(())
}

fn handle_notification(not: Notification) -> Result<(), Box<dyn Error + Sync + Send>> {
    eprintln!("Received notification: {:?}", not);
    Ok(())
}

fn handle_completion(_params: CompletionParams) -> Vec<CompletionItem> {
    let mut items = vec![];
    items.push(CompletionItem {
        label: "Hello, World!".to_string(),
        kind: Some(CompletionItemKind::TEXT),
        detail: Some("This is a test completion.".to_string()),
        ..Default::default()
    });
    items
}

fn cast<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}
