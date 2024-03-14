use crossbeam_channel::{Receiver, select};
use lsp_server::{Connection, ErrorCode, Message, Notification, Request, Response};
use lsp_types::{
    CompletionOptions, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind,
};
use std::time::Instant;
use serde::de::DeserializeOwned;

mod config;
use config::Config;

mod global_state;
use global_state::GlobalState;

mod dispatcher;
use dispatcher::RequestDispatcher;

use crate::dispatcher::NotificationDispatcher;

mod lsp;

mod handlers;

enum Event {
    Lsp(Message),
}
fn main() -> anyhow::Result<()> {
    eprintln!("Starting ColdFusion Language Server...");

    let (connection, io_threads) = Connection::stdio();

    let (initialize_id, initialize_params) = match connection.initialize_start() {
        Ok(it) => it,
        Err(e) => {
            if e.channel_is_disconnected() {
                io_threads.join()?;
            }
            return Err(e.into());
        }
    };

    let lsp_types::InitializeParams {
        root_uri,
        initialization_options,
        capabilities,
        workspace_folders,
        ..
    } = from_json::<lsp_types::InitializeParams>("InitializeParams", &initialize_params)?;

    let root_path = root_uri
        .and_then(|it| it.to_file_path().ok())
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let workspace_roots = workspace_folders
        .map(|workspaces| {
            workspaces
                .iter()
                .filter_map(|it| it.uri.to_file_path().ok())
                .collect::<Vec<_>>()
        }).filter(|it| !it.is_empty())
    .unwrap_or_else(|| vec![root_path.clone()]);

    let mut config = Config::new(root_path, capabilities, workspace_roots);

    if let Some(json) = initialization_options {
        if let Err(e) = config.update(json) {
            use lsp_types::{
               notification::{Notification, ShowMessage},
               MessageType, ShowMessageParams
            };

            let notification = lsp_server::Notification::new(ShowMessage::METHOD.to_owned(), ShowMessageParams {
                typ: MessageType::WARNING,
                message: format!("Failed to update configuration: {:?}", e),
            });
            connection.sender.send(Message::Notification(notification)).unwrap();
        }
    }

    let server_capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        completion_provider: Some(CompletionOptions {
            resolve_provider: Some(true),
            trigger_characters: Some(vec![".".to_string()]),
            work_done_progress_options: Default::default(),
            all_commit_characters: None,
            completion_item: None,
        }),
        ..ServerCapabilities::default()
    };

    let initialize_result = lsp_types::InitializeResult {
        capabilities: server_capabilities,
        server_info: Some(
            lsp_types::ServerInfo {
                name: "ColdFusion Language Server".to_string(),
                version: Some("0.1.0".to_string()),
            }
        ),
    };

    let initialize_result = serde_json::to_value(initialize_result).unwrap();

    if let Err(e) = connection.initialize_finish(initialize_id, initialize_result) {
        if e.channel_is_disconnected() {
            io_threads.join()?;
        }
        return Err(e.into());
    }

    run(config, connection)?;
    io_threads.join()?;
    eprintln!("ColdFusion Language Server has stopped.");
    Ok(())
}

fn run(
    config: Config,
    connection: Connection,
) -> anyhow::Result<()> {
    #[cfg(windows)]

    unsafe {
        use winapi::um::processthreadsapi::*;
        let thread = GetCurrentThread();
        let thread_priority_above_normal = 1;
        SetThreadPriority(thread, thread_priority_above_normal);
    }

    GlobalState::new(connection.sender, config).run(connection.receiver)
}

impl GlobalState {
    fn run(mut self, inbox: Receiver<Message>) -> anyhow::Result<()> {
        while let Some(event) = self.next_event(&inbox) {
            if matches!(
                &event,
                Event::Lsp(Message::Notification(Notification { method, ..}))
                if method == "exit"
            ) {
                return Ok(());
                }

            self.handle_event(event)?;
        }

        anyhow::bail!("Connection was terminated")
    }

    fn next_event(&self, inbox: &Receiver<Message>) -> Option<Event> {
        select! {
            recv(inbox) -> msg => msg.ok().map(Event::Lsp),
        }
    }

    fn handle_event(&mut self, event: Event) -> anyhow::Result<()> {
        let loop_start = Instant::now();
        match event {
            Event::Lsp(msg) => match msg {
                Message::Request(req) => self.on_new_request(loop_start, req),
                Message::Notification(notification) => self.on_notification(notification)?,
                Message::Response(resp) => self.complete_request(resp),
            }
        }

        let event_duration = loop_start.elapsed();
        Ok(())
    }

    fn on_new_request(&mut self, request_received: Instant, req: Request) {
        self.register_request(&req, request_received);
        self.on_request(req);
    }

    fn on_request(&mut self, req: Request) {
        let mut dispatcher = RequestDispatcher {
            req: Some(req),
            global_state: self,
        };

        dispatcher.on_sync_mut::<lsp_types::request::Shutdown>(|s, ()| {
            s.shutdown_requested = true;
            Ok(())
        });

        match &mut dispatcher {
            RequestDispatcher { req: Some(req), global_state: this } if this.shutdown_requested => {
                let invalid_request = ErrorCode::InvalidRequest as i32;
                this.respond(Response::new_err(req.id.clone(), invalid_request, "Shutdown already requested".to_owned()));
            }
            _ => ()
        };

        use handlers::request as handlers;
        use lsp_types::request as lsp_request;

        dispatcher.on_sync_mut::<lsp_request::Completion>(handlers::handle_completion)
        .finish();

    }

    fn on_notification(&mut self, notification: Notification) -> anyhow::Result<()> {
        use handlers::notifications as handlers;
        use lsp_types::notification as notifs;

        let mut dispatcher = NotificationDispatcher {
            notification: Some(notification),
            global_state: self,
        };

        dispatcher.on_sync_mut::<notifs::Cancel>(handlers::handle_cancel)?
        .finish();
        Ok(())
    }
}

pub fn from_json<T: DeserializeOwned>(
    what: &'static str,
    json: &serde_json::Value,
) -> anyhow::Result<T> {
    serde_json::from_value(json.clone()).map_err(|e| {
        anyhow::anyhow!(
            "Failed to deserialize {} from JSON: {}\n{}",
            what,
            e,
            json
        )
    })
}
