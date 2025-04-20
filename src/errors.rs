use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::warn;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("response: {0:?}")]
    Response(Response),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            ServerError::Response(response) =>
                response,

            ServerError::Other(inner) => {
                warn!("other error: {:#?}", inner);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

impl From<StatusCode> for ServerError {
    fn from(value: StatusCode) -> Self {
        Self::Response(value.into_response())
    }
}

impl From<Response> for ServerError {
    fn from(value: Response) -> Self {
        Self::Response(value)
    }
}
