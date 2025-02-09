use serde::Serialize;

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct InfoResponse {
    pub version: String,
    pub client_id: String,
    pub token_url: String,
    pub authorize_url: String,
}
