use lsp_types::{CancelParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidChangeTextDocumentParams};

use crate::global_state::GlobalState;

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

    state.add_document(uri, text);
    Ok(())
}


pub(crate) fn handle_did_close_text_document(state: &mut GlobalState, params: DidCloseTextDocumentParams) -> anyhow::Result<()> {
	let _p = tracing::span!(tracing::Level::DEBUG, "handle_did_close_text_document").entered();
	let text_document = params.text_document;
	state.remove_document(text_document.uri);
	Ok(())
}

pub(crate) fn handle_did_change_text_document(
	state: &mut GlobalState,
	params: DidChangeTextDocumentParams,
) -> anyhow::Result<()> {
	let _p = tracing::span!(tracing::Level::DEBUG, "handle_did_change_text_document").entered();
	let text_document = params.text_document;
	state.add_document(text_document.uri, params.content_changes[0].text.clone());
	Ok(())
}