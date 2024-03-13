use lsp_types::{CompletionItem, CompletionItemKind, CompletionParams};

pub fn handle_completion(_params: CompletionParams) -> Vec<CompletionItem> {
    let mut items = vec![];
    items.push(CompletionItem {
        label: "Hello, World!".to_string(),
        kind: Some(CompletionItemKind::TEXT),
        detail: Some("This is a test completion.".to_string()),
        ..Default::default()
    });
    items
}
