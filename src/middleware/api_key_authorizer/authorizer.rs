use axum::async_trait;
use axum::extract::FromRequestParts;
use http::request::Parts;
use crate::entities::sea_orm_active_enums::ApiKeyType;
use crate::middleware::error::AuthError;
use crate::middleware::helpers::{extract_api_key, extract_organization_id, find_and_validate_key};
use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct ApiKeyAuthorizer {
    pub key_type: ApiKeyType,
}

#[async_trait]
impl FromRequestParts<AppState> for ApiKeyAuthorizer {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let org_id = extract_organization_id(parts, state).await?;
        let api_key = extract_api_key(parts)?;
        let key = find_and_validate_key(&api_key, &org_id, &state.db.connection).await?;

        Ok(Self {
            key_type: key.key_type,
        })
    }
}
