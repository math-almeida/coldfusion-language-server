use lsp_types::{
    CancelParams, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams,
};

use crate::global_state::GlobalState;
use crate::global_state::DocumentData;

pub(crate) fn handle_cancel(state: &mut GlobalState, params: CancelParams) -> anyhow::Result<()> {
    let id = match params.id {
        lsp_types::NumberOrString::Number(id) => id.into(),
        lsp_types::NumberOrString::String(id) => id.into(),
    };
    state.cancel(id);
    Ok(())
}

pub(crate) fn handle_did_open_text_document(
    state: &mut GlobalState,
    params: DidOpenTextDocumentParams,
) -> anyhow::Result<()> {
    let _p = tracing::span!(tracing::Level::DEBUG, "handle_did_open_text_document").entered();
    let text_document = params.text_document;
    let uri = text_document.uri;
    let text = text_document.text;
    let version = text_document.version;

    let already_exists = state.add_document(&uri.clone(), text.clone(), version).is_err();
    if already_exists {
        tracing::error!("duplicate didOpen notification for {:?}", uri);
    }

    state.add_changes_into_document(&uri, text);
    Ok(())
}

pub(crate) fn handle_did_close_text_document(
    state: &mut GlobalState,
    params: DidCloseTextDocumentParams,
) -> anyhow::Result<()> {
    let _p = tracing::span!(tracing::Level::DEBUG, "handle_did_close_text_document").entered();
    let text_document = params.text_document;
    if state.remove_document(&text_document.uri).is_err() {
        tracing::error!("didClose notification for non-existing file: {:?}", text_document.uri);
    }

    Ok(())
}

pub(crate) fn handle_did_change_text_document(
    state: &mut GlobalState,
    params: DidChangeTextDocumentParams,
) -> anyhow::Result<()> {
    let _p = tracing::span!(tracing::Level::DEBUG, "handle_did_change_text_document").entered();
    let text_document = params.text_document;
    let content_changes = params.content_changes;
    let uri = text_document.uri;
    let DocumentData { data, version } = if let Some(doc) = state.get_document(&uri) {
        doc
    } else {
        tracing::error!("didChange notification for non-existing file: {:?}", uri);
        return Ok(());
    };
    Ok(())
}
