use actix_http::ResponseBuilder;
use actix_web::error::ResponseError;
use actix_web::{HttpRequest, HttpResponse};
use serde::Serialize;
use std::fmt::Display;
use std::fmt::Formatter;

use actix_web::error::{Error as ActixError, JsonPayloadError};

#[derive(Debug, Serialize)]
pub struct ServerErrorResponse {
    pub name: String,
    pub message: Option<String>,
}

#[derive(Debug)]
pub enum ServerError {
    BadRequest(String),
    NotFound(String),
    InternalServerError(String),
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "InternalServerError")
    }
}

impl ServerError {
    fn response(&self) -> ResponseBuilder {
        match *self {
            ServerError::BadRequest(_) => HttpResponse::BadRequest(),
            ServerError::NotFound(_) => HttpResponse::NotFound(),
            ServerError::InternalServerError(_) => HttpResponse::InternalServerError(),
        }
    }

    fn name(&self) -> String {
        match *self {
            ServerError::BadRequest(_) => "Bad Request".into(),
            ServerError::NotFound(_) => "Not Found".into(),
            ServerError::InternalServerError(_) => "Internal Server Error".into(),
        }
    }

    fn message(&self) -> Option<String> {
        match *self {
            ServerError::BadRequest(ref msg) => Some(msg.clone()),
            ServerError::NotFound(ref msg) => Some(msg.clone()),
            ServerError::InternalServerError(ref msg) => Some(msg.clone()),
        }
    }
}

impl ResponseError for ServerError {
    fn error_response(&self) -> HttpResponse {
        self.response().json(ServerErrorResponse {
            name: self.name(),
            message: self.message(),
        })
    }
}

impl From<JsonPayloadError> for ServerError {
    fn from(error: JsonPayloadError) -> Self {
        match error {
            JsonPayloadError::Deserialize(err) => ServerError::BadRequest(err.to_string()),
            _ => ServerError::BadRequest(error.to_string()),
        }
    }
}

impl std::convert::From<r2d2::Error> for ServerError {
    fn from(error: r2d2::Error) -> Self {
        ServerError::InternalServerError(error.to_string())
    }
}

pub fn json_error_handler(err: JsonPayloadError, _req: &HttpRequest) -> ActixError {
    error!("json_error_handler: {:?}", err);
    let error = ServerError::from(err);
    let res = error.error_response();
    actix_web::error::InternalError::from_response(error, res).into()
}

impl std::convert::From<std::io::Error> for ServerError {
    fn from(error: std::io::Error) -> Self {
        ServerError::InternalServerError(error.to_string())
    }
}
