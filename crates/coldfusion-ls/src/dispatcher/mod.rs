use lsp_server::{ErrorCode, ExtractError, Notification, Request, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt, panic};

use crate::{
    from_json,
    global_state::GlobalState,
    lsp::{Cancelled, LspError},
};

pub struct RequestDispatcher<'a> {
    pub req: Option<Request>,
    pub global_state: &'a mut GlobalState,
}

impl RequestDispatcher<'_> {
    fn parse<R>(&mut self) -> Option<(Request, R::Params, String)>
    where
        R: lsp_types::request::Request,
        R::Params: DeserializeOwned + fmt::Debug,
    {
        let req = match &self.req {
            Some(req) if req.method == R::METHOD => self.req.take()?,
            _ => return None,
        };

        let res = from_json(R::METHOD, &req.params);
        match res {
            Ok(params) => {
                let panic_context = format!("{}::{} {params:#?}", R::METHOD, req.id);
                Some((req, params, panic_context))
            }
            Err(err) => {
                let invalid_params = ErrorCode::InvalidParams as i32;
                self.global_state.respond(Response::new_err(
                    req.id,
                    invalid_params,
                    err.to_string(),
                ));
                None
            }
        }
    }
    pub(crate) fn finish(&mut self) {
        if let Some(req) = self.req.take() {
            tracing::error!("unknown request: {:?}", req);
            let response = lsp_server::Response::new_err(
                req.id,
                lsp_server::ErrorCode::MethodNotFound as i32,
                "unknown request".to_owned(),
            );
            self.global_state.respond(response);
        }
    }
    pub(crate) fn on_sync_mut<R>(
        &mut self,
        f: fn(&mut GlobalState, R::Params) -> anyhow::Result<R::Result>,
    ) -> &mut Self
    where
        R: lsp_types::request::Request,
        R::Params: DeserializeOwned + panic::UnwindSafe + fmt::Debug,
        R::Result: Serialize,
    {
        let (req, params, _panic_context) = match self.parse::<R>() {
            Some(it) => it,
            None => return self,
        };
        let _guard = tracing::span!(tracing::Level::INFO, "request", method = ?req.method, "request_id" = ?req.id).entered();
        tracing::debug!(?params);
        let result = { f(self.global_state, params) };
        if let Ok(response) = result_to_response::<R>(req.id, result) {
            self.global_state.respond(response);
        }

        self
    }
}

pub struct NotificationDispatcher<'a> {
    pub notification: Option<Notification>,
    pub global_state: &'a mut GlobalState,
}

impl NotificationDispatcher<'_> {
    pub(crate) fn finish(&mut self) {
        if let Some(notification) = &self.notification {
            if !notification.method.starts_with("$/") {
                tracing::error!("unknown notification: {:?}", notification);
            }
        }
    }

    pub(crate) fn on_sync_mut<N>(
        &mut self,
        f: fn(&mut GlobalState, N::Params) -> anyhow::Result<()>,
    ) -> anyhow::Result<&mut Self>
    where
        N: lsp_types::notification::Notification,
        N::Params: DeserializeOwned + Send + fmt::Debug,
    {
        let notification = match self.notification.take() {
            Some(it) => it,
            None => return Ok(self),
        };

        let params = match notification.extract::<N::Params>(N::METHOD) {
            Ok(it) => it,
            Err(ExtractError::JsonError { method, error }) => {
                panic!("Invalid request: {} {}", method, error)
            }
            Err(ExtractError::MethodMismatch(notification)) => {
                self.notification = Some(notification);
                return Ok(self);
            }
        };

        f(self.global_state, params)?;
        Ok(self)
    }
}
fn result_to_response<R>(
    id: lsp_server::RequestId,
    result: anyhow::Result<R::Result>,
) -> Result<lsp_server::Response, Cancelled>
where
    R: lsp_types::request::Request,
    R::Params: DeserializeOwned,
    R::Result: Serialize,
{
    let res = match result {
        Ok(res) => lsp_server::Response::new_ok(id, &res),
        Err(e) => match e.downcast::<LspError>() {
            Ok(lsp_error) => lsp_server::Response::new_err(id, lsp_error.code, lsp_error.message),
            Err(e) => match e.downcast::<Cancelled>() {
                Ok(cancelled) => return Err(cancelled),
                Err(e) => {
                    let code = ErrorCode::InternalError as i32;
                    Response::new_err(id, code, e.to_string())
                }
            },
        },
    };
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::request::{Shutdown, Request};
    use crate::config::Config;
    use lsp_server::Request as LspRequest;

    #[test]
    fn test_request_dispatcher() {
        let (sender, _) = crossbeam_channel::unbounded();
        let mut global_state = GlobalState::new(sender, Config::default());
        let mut dispatcher = RequestDispatcher {
            req: Some(LspRequest {
                id: lsp_server::RequestId::from(1),
                method: Shutdown::METHOD.to_string(),
                params: serde_json::Value::Null,
            }),
            global_state: &mut global_state,
        };

        dispatcher.finish();

        assert!(dispatcher.req.is_none());
    }

    #[test]
    fn test_notification_dispatcher() {
        let (sender, _) = crossbeam_channel::unbounded();
        let mut global_state = GlobalState::new(sender, Config::default());
        let mut dispatcher = NotificationDispatcher {
            notification: Some(Notification {
                method: "textDocument/didOpen".to_string(),
                params: serde_json::Value::Null,
            }),
            global_state: &mut global_state,
        };

        dispatcher.finish();

        assert!(dispatcher.notification.is_some());
    }

    #[test]
    fn test_result_to_response() {
        let id = lsp_server::RequestId::from(1);
        let result = Ok(());
        let response = result_to_response::<Shutdown>(id, result);
        assert!(response.is_ok());
    }

}
