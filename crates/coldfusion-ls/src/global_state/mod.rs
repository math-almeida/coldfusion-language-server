use crossbeam_channel::Sender;
use lsp_server::{Message, Request, Response};
use std::sync::Arc;
use std::time::Instant;

use crate::config::Config;

type ReqHandler = fn(&mut GlobalState, lsp_server::Response);
type ReqQueue = lsp_server::ReqQueue<(String, Instant), ReqHandler>;

pub struct GlobalState {
    sender: Sender<Message>,
    pub config: Arc<Config>,
    req_queue: ReqQueue,
    pub shutdown_requested: bool,
}

pub(crate) struct GlobalStateSnapshot {
    pub(crate) config: Arc<Config>,
}

impl std::panic::UnwindSafe for GlobalStateSnapshot {}

impl GlobalState {
    pub fn new(sender: Sender<Message>, config: Config) -> Self {
        GlobalState {
            sender,
            config: Arc::new(config.clone()),
            req_queue: ReqQueue::default(),
            shutdown_requested: false,
        }
    }

    pub fn register_request(&mut self, request: &Request, request_received: Instant) {
        self.req_queue.incoming.register(
            request.id.clone(),
            (request.method.clone(), request_received),
        );
    }

    pub(crate) fn snapshot(&self) -> GlobalStateSnapshot {
        GlobalStateSnapshot {
            config: self.config.clone(),
        }
    }

    pub(crate) fn respond(&mut self, response: lsp_server::Response) {
        if let Some((method, start)) = self.req_queue.incoming.complete(response.id.clone()) {
            if let Some(e) = &response.error {
                if e.message.starts_with("server panicked") {
                    tracing::error!("server panicked while handling request: {:?}", method);
                }
            }
        let duration = start.elapsed();
        tracing::debug!("handled request {} in {:0.2?}", method, duration);
        self.send(response.into())
        }
    }

    fn send(&self, message: lsp_server::Message) {
        self.sender.send(message).unwrap()
    }

    pub fn cancel(&mut self, request_id: lsp_server::RequestId) {
        if let Some(response) = self.req_queue.incoming.cancel(request_id) {
            self.send(response.into());
        }
    }

    pub(crate) fn complete_request(&mut self, response: Response) {
        let handler = self.req_queue.outgoing.complete(response.id.clone()).expect("received response for unknown request");

        handler(self, response);
    }
}
