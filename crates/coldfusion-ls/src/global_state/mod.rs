use crossbeam_channel::Sender;
use lsp_server::{Message, Request, Response};
use lsp_types::Url;
use nohash_hasher::IntMap;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Instant;
use virtual_fs::{FileId, VirtualFS};

use crate::config::Config;
mod from_proto;
mod line_index;
mod mem_docs;

type ReqHandler = fn(&mut GlobalState, lsp_server::Response);
type ReqQueue = lsp_server::ReqQueue<(String, Instant), ReqHandler>;
pub(crate) use mem_docs::DocumentData;
pub use mem_docs::MemDocs;

use self::line_index::LineEndings;

pub struct GlobalState {
    sender: Sender<Message>,
    pub config: Arc<Config>,
    req_queue: ReqQueue,
    pub shutdown_requested: bool,
    mem_docs: MemDocs,
    vfs: Arc<RwLock<(VirtualFS, IntMap<FileId, LineEndings>)>>,
}

pub(crate) struct GlobalStateSnapshot {
    pub(crate) config: Arc<Config>,
    pub(crate) mem_docs: MemDocs,
    vfs: Arc<RwLock<(VirtualFS, IntMap<FileId, LineEndings>)>>,
}

impl std::panic::UnwindSafe for GlobalStateSnapshot {}

impl GlobalState {
    pub fn new(sender: Sender<Message>, config: Config) -> Self {
        GlobalState {
            sender,
            config: Arc::new(config.clone()),
            req_queue: ReqQueue::default(),
            shutdown_requested: false,
            mem_docs: MemDocs::default(),
            vfs: Arc::new(RwLock::new((VirtualFS::default(), IntMap::default()))),
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
            config: Arc::clone(&self.config),
            mem_docs: self.mem_docs.clone(),
            vfs: Arc::clone(&self.vfs),
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
        let handler = self
            .req_queue
            .outgoing
            .complete(response.id.clone())
            .expect("received response for unknown request");

        handler(self, response);
    }

    pub(crate) fn add_document(
        &mut self,
        uri: &Url,
        text: String,
        version: i32,
    ) -> anyhow::Result<()> {
        let path = from_proto::vfs_path(uri).unwrap();
        let data = mem_docs::DocumentData::new(version, text.into_bytes());
        self.mem_docs.insert(path, data).unwrap();
        Ok(())
    }

    pub(crate) fn remove_document(&mut self, uri: &Url) -> anyhow::Result<()> {
        let path = from_proto::vfs_path(uri).unwrap();
        self.mem_docs.remove(&path).unwrap();
        Ok(())
    }

    pub(crate) fn get_document(&self, uri: &Url) -> Option<&mem_docs::DocumentData> {
        let path = match from_proto::vfs_path(uri) {
            Ok(it) => it,
            Err(_) => return None,
        };
        self.mem_docs.get(&path)
    }

    pub(crate) fn add_changes_into_document(&mut self, uri: &Url, text: String) {
        let path = from_proto::vfs_path(uri).unwrap();
        self.vfs
            .write()
            .0
            .set_file_contents(path, Some(text.into_bytes()));
    }
}
