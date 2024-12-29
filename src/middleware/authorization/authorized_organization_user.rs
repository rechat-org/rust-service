use crate::entities::{organization_members, prelude::*, sea_orm_active_enums::OrganizationRole};
use crate::middleware::authorization::auth::AuthUser;
use crate::middleware::error::AuthError;
use crate::middleware::helpers::extract_organization_id;
use crate::state::AppState;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use sea_orm::*;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthorizedOrganizationUser {
    pub user: AuthUser,
    pub organization_id: Uuid,
    pub role: OrganizationRole,
}

impl AuthorizedOrganizationUser {
    pub fn can_manage_api_keys(&self) -> bool {
        matches!(
            self.role,
            OrganizationRole::Owner | OrganizationRole::Admin | OrganizationRole::Developer
        )
    }
}

#[async_trait]
impl FromRequestParts<AppState> for AuthorizedOrganizationUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;
        let organization_id = extract_organization_id(parts, state).await?;

        // Verify user has access to this organization
        let member = OrganizationMembers::find()
            .filter(
                organization_members::Column::UserId
                    .eq(user.user_id)
                    .and(organization_members::Column::OrganizationId.eq(organization_id)),
            )
            .one(&state.db.connection)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        match member {
            Some(member) => Ok(AuthorizedOrganizationUser {
                user,
                organization_id,
                role: member.role,
            }),
            None => Err(AuthError::InsufficientPermissions),
        }
    }
}

pub struct ApiKeyManager(pub AuthorizedOrganizationUser);

#[async_trait]
impl FromRequestParts<AppState> for ApiKeyManager {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = AuthorizedOrganizationUser::from_request_parts(parts, state).await?;

        if !auth_user.can_manage_api_keys() {
            return Err(AuthError::InsufficientPermissions);
        }

        Ok(Self(auth_user))
    }
}
