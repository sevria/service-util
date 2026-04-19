use axum::{
    extract::{FromRequest, rejection::JsonRejection},
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::error::Error;

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(Error))]
pub struct Json<T>(pub T);

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        let Self(value) = self;
        axum::Json(value).into_response()
    }
}

impl From<JsonRejection> for Error {
    fn from(err: JsonRejection) -> Self {
        Error::BadRequest(err.to_string())
    }
}
