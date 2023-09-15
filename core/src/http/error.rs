use actix_web::{HttpResponse, ResponseError};
use actix_web::http::StatusCode;

use crate::http::api::Error;

impl ResponseError for crate::Error {
    fn status_code(&self) -> StatusCode {
        match self {
            crate::Error::InvalidArgument(_) => StatusCode::BAD_REQUEST,
            crate::Error::AuthenticationRequired => StatusCode::UNAUTHORIZED,
            crate::Error::PermissionDenied(_) => StatusCode::FORBIDDEN,
            crate::Error::NotFound(_) => StatusCode::NOT_FOUND,
            crate::Error::AlreadyExists(_) => StatusCode::CONFLICT,
            crate::Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let err: Error = self.clone().into();
        HttpResponse::build(self.status_code()).json(err)
    }
}