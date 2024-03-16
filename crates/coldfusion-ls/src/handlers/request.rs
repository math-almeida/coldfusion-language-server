use crate::global_state::GlobalState;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use lsp_types::CompletionParams;
    use lsp_types::Position;
    use lsp_types::TextDocumentIdentifier;
    use lsp_types::TextDocumentPositionParams;
    use lsp_types::Url;
    use lsp_types::WorkDoneProgressParams;
    use virtual_fs::AbsPathBuf;

    #[test]
    fn test_handle_completion() {
        let (sender, _) = crossbeam_channel::unbounded();
        let root_path = AbsPathBuf::try_from("/tmp").unwrap();
        let capabilities = lsp_types::ClientCapabilities::default();
        let workspace_roots = vec![AbsPathBuf::try_from("/tmp").unwrap()];
        let config = Config::new(root_path, capabilities, workspace_roots);
        let mut snap = GlobalState::new(sender, config);
        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: Url::parse("file:///tmp/test.txt").unwrap(),
                },
                position: Position {
                    line: 0,
                    character: 0,
                },
            },
            context: None,
            work_done_progress_params: WorkDoneProgressParams {
                work_done_token: None,
            },
            partial_result_params: lsp_types::PartialResultParams {
                partial_result_token: None,
            },
        };
        let result = handle_completion(&mut snap, params);
        assert!(result.is_ok());
    }
}
