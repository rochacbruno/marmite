use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use std::fmt;

pub enum AppError {
    SessionNotFound(uuid::Uuid),
    FileNotFound(String),
    PathTraversal(String),
    RenderFailed(String),
    Forbidden,
    Io(std::io::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SessionNotFound(id) => write!(f, "session not found: {id}"),
            Self::FileNotFound(path) => write!(f, "file not found: {path}"),
            Self::PathTraversal(path) => write!(f, "invalid path: {path}"),
            Self::RenderFailed(msg) => write!(f, "render failed: {msg}"),
            Self::Forbidden => write!(f, "forbidden: not the session owner"),
            Self::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::SessionNotFound(_) | Self::FileNotFound(_) => StatusCode::NOT_FOUND,
            Self::PathTraversal(_) => StatusCode::BAD_REQUEST,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::RenderFailed(_) | Self::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = serde_json::json!({ "error": self.to_string() });
        (status, Json(body)).into_response()
    }
}
