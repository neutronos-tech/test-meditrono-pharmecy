use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
pub enum AppError {
    Database(sqlx_core::error::Error),
    Template(askama::Error),
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    PaymentRequired,
    Conflict,
    Unavailable,
    Internal,
}

impl From<sqlx_core::error::Error> for AppError {
    fn from(error: sqlx_core::error::Error) -> Self {
        Self::Database(error)
    }
}

impl From<askama::Error> for AppError {
    fn from(error: askama::Error) -> Self {
        Self::Template(error)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::BadRequest => (StatusCode::BAD_REQUEST, "Invalid request.").into_response(),
            Self::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "Authentication required.").into_response()
            }
            Self::Forbidden => (StatusCode::FORBIDDEN, "Request rejected.").into_response(),
            Self::NotFound => (StatusCode::NOT_FOUND, "Not found.").into_response(),
            Self::Conflict => (StatusCode::CONFLICT, "That username is taken.").into_response(),
            Self::PaymentRequired => (
                StatusCode::PAYMENT_REQUIRED,
                "Subscription payment required.",
            )
                .into_response(),
            Self::Unavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Licensing is not configured.",
            )
                .into_response(),
            Self::Database(error) => {
                tracing::error!(%error, "database operation failed");
                (StatusCode::INTERNAL_SERVER_ERROR, "Request failed.").into_response()
            }
            Self::Template(error) => {
                tracing::error!(%error, "template rendering failed");
                (StatusCode::INTERNAL_SERVER_ERROR, "Page rendering failed.").into_response()
            }
            Self::Internal => {
                tracing::error!("internal operation failed");
                (StatusCode::INTERNAL_SERVER_ERROR, "Request failed.").into_response()
            }
        }
    }
}
