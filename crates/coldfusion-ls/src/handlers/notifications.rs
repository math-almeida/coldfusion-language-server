use lsp_types::CancelParams;

use crate::global_state::GlobalState;

pub(crate) fn handle_cancel(state: &mut GlobalState, params: CancelParams) -> anyhow::Result<()> {
    let id = match params.id {
        lsp_types::NumberOrString::Number(id) => id.into(),
        lsp_types::NumberOrString::String(id) => id.into(),
    };
    state.cancel(id);
    Ok(())
}
