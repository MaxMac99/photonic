use axum::http::StatusCode;

pub async fn ping() -> StatusCode {
    StatusCode::NO_CONTENT
}
