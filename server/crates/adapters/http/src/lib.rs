//! HTTP inbound adapter: Axum routers, API DTOs and JWT authentication.

pub mod api;
pub mod auth;
pub mod settings;

pub use auth::JwtUserClaims;
pub use settings::AuthSettings;
