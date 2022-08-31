use crate::app::SanitizedError;
use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::{self, Responder, Response};

impl<'r> Responder<'r, 'static> for SanitizedError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let status_code = match self {
            SanitizedError::NotFound(_) => Status::NotFound,
            SanitizedError::Unauthorized => Status::Forbidden,
            SanitizedError::Contention | SanitizedError::InternalError => Status::InternalServerError,
            SanitizedError::IncorrectRevisionNumber => Status::Conflict,
            SanitizedError::UserError(_) => Status::BadRequest,
        };

        Response::build_from(self.to_string().respond_to(req)?)
            .status(status_code)
            .header(ContentType::new("text", "plain"))
            .ok()
    }
}
