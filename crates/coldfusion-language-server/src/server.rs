use std::error::Error;

use lsp_server::{Connection, ExtractError, Message, Notification, Request, RequestId, Response};

use lsp_types::request::Completion;

use completion;

pub fn handle_request(
    req: Request,
    connection: &Connection,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    if connection.handle_shutdown(&req)? {
        return Ok(());
    }
    match cast::<Completion>(req) {
        Ok((id, params)) => {
            let result = completion::handle_completion(params);
            let resp = Response::new_ok(id, result);
            connection.sender.send(Message::Response(resp))?;
            Ok(())
        }
        Err(err @ ExtractError::JsonError { .. }) => panic!("JSON error: {}", err),
        Err(ExtractError::MethodMismatch(req)) => {
            println!("Method mismatch: {:?}", req);
            Ok(())
        }
    }
}

pub fn handle_response(resp: Response) -> Result<(), Box<dyn Error + Sync + Send>> {
    eprintln!("Received response: {:?}", resp);
    Ok(())
}

pub fn handle_notification(not: Notification) -> Result<(), Box<dyn Error + Sync + Send>> {
    eprintln!("Received notification: {:?}", not);
    Ok(())
}

fn cast<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}
