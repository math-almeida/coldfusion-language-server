use crate::global_state::{GlobalState};
use lsp_types::{CompletionItemKind, CompletionParams};

pub fn handle_completion(
    _snap: &mut GlobalState,
    _params: CompletionParams,
) -> anyhow::Result<Option<lsp_types::CompletionResponse>> {
    let completion_list = lsp_types::CompletionList {
        is_incomplete: false,
        items: vec![lsp_types::CompletionItem {
            label: "Hello, World!".to_string(),
            kind: Some(CompletionItemKind::TEXT),
            detail: Some("This is a completion item".to_string()),
            ..Default::default()
        }],
    };
    Ok(Some(completion_list.into()))

}
