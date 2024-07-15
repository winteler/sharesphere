use std::collections::HashMap;
use std::env;

#[cfg(feature = "ssr")]
use axum_session_auth::Authentication;
use leptos::*;
use leptos_router::*;
#[cfg(feature = "ssr")]
use openidconnect as oidc;
#[cfg(feature = "ssr")]
use openidconnect::{OAuth2TokenResponse, TokenResponse};
#[cfg(feature = "ssr")]
use openidconnect::reqwest::*;
use serde::{Deserialize, Serialize};

use crate::app::GlobalState;
#[cfg(feature = "ssr")]
use crate::app::ssr::{get_db_pool, get_session};
#[cfg(feature = "ssr")]
use crate::auth::ssr::{check_user, create_user, SqlUser};
use crate::errors::AppError;
use crate::navigation_bar::get_current_path;
use crate::role::{AdminRole, PermissionLevel};
use crate::unpack::SuspenseUnpack;

pub const BASE_URL_ENV: &str = "LEPTOS_SITE_ADDR";
pub const OIDC_ISSUER_URL_ENV: &str = "OIDC_ISSUER_ADDR";
pub const AUTH_CLIENT_ID_ENV: &str = "AUTH_CLIENT_ID";
pub const AUTH_CLIENT_SECRET_ENV: &str = "AUTH_CLIENT_SECRET";
pub const AUTH_CALLBACK_ROUTE: &str = "/authback";
pub const PKCE_KEY: &str = "pkce";
pub const NONCE_KEY: &str = "nonce";
pub const OIDC_TOKENS_KEY: &str = "oidc_token";
pub const OIDC_USERNAME_KEY: &str = "oidc_username";
pub const REDIRECT_URL_KEY: &str = "redirect";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum BanStatus {
    None,
    Until(chrono::DateTime<chrono::Utc>),
    Permanent,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub user_id: i64,
    pub oidc_id: String,
    pub username: String,
    pub email: String,
    pub admin_role: AdminRole,
    pub permission_by_forum_map: HashMap<String, PermissionLevel>,
    pub ban_status: BanStatus,
    pub ban_status_by_forum_map: HashMap<String, BanStatus>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub is_deleted: bool,
}

impl BanStatus {
    pub fn is_permanent(&self) -> bool {
        *self == BanStatus::Permanent
    }
    pub fn is_active(&self) -> bool {
        match self {
            BanStatus::Until(until_timestamp) => *until_timestamp > chrono::offset::Utc::now(),
            _ => self.is_permanent(),
        }
    }
}

impl Default for User {
    fn default() -> Self {
        Self {
            user_id: -1,
            oidc_id: String::default(),
            username: String::default(),
            email: String::default(),
            admin_role: AdminRole::None,
            permission_by_forum_map: HashMap::new(),
            ban_status: BanStatus::None,
            ban_status_by_forum_map: HashMap::new(),
            timestamp: chrono::DateTime::default(),
            is_deleted: false,
        }
    }
}

impl User {
    fn check_admin_role(&self, req_admin_role: AdminRole) -> Result<(), AppError> {
        match self.admin_role >= req_admin_role {
            true => Ok(()),
            false => Err(AppError::InsufficientPrivileges),
        }
    }

    fn check_forum_permissions(&self, forum_name: &str, req_permission_level: PermissionLevel) -> Result<(), AppError> {
        match self.permission_by_forum_map.get(forum_name).is_some_and(|permission_level| *permission_level >= req_permission_level) {
            true => Ok(()),
            false => Err(AppError::InsufficientPrivileges)
        }
    }

    fn check_permissions(&self, req_admin_role: AdminRole, forum_name: &str, req_permission_level: PermissionLevel) -> Result<(), AppError> {
        match self.check_admin_role(req_admin_role).is_ok() || self.check_forum_permissions(forum_name, req_permission_level).is_ok() {
            true => Ok(()),
            false => Err(AppError::InsufficientPrivileges)
        }
    }
    pub fn check_is_global_moderator(&self) -> Result<(), AppError> {
        self.check_admin_role(AdminRole::Moderator)
    }

    pub fn check_is_admin(&self) -> Result<(), AppError> {
        self.check_admin_role(AdminRole::Admin)
    }

    pub fn check_can_moderate_forum(&self, forum_name: &str) -> Result<(), AppError> {
        self.check_permissions(AdminRole::Moderator, forum_name, PermissionLevel::Moderate)
    }

    pub fn check_can_ban_users(&self, forum_name: &str) -> Result<(), AppError> {
        self.check_permissions(AdminRole::Moderator, forum_name, PermissionLevel::Ban)
    }

    pub fn check_can_manage_forum(&self, forum_name: &str) -> Result<(), AppError> {
        self.check_permissions(AdminRole::Admin, forum_name, PermissionLevel::Manage)
    }

    pub fn check_is_forum_leader(&self, forum_name: &str) -> Result<(), AppError> {
        self.check_forum_permissions(forum_name, PermissionLevel::Lead)
    }

    pub fn check_can_publish(&self) -> Result<(), AppError> {
        match self.ban_status.is_active() {
            true => match self.ban_status {
                BanStatus::Until(timestamp) => Err(AppError::GlobalBanUntil(timestamp)),
                BanStatus::Permanent => Err(AppError::PermanentGlobalBan),
                BanStatus::None => Err(AppError::InternalServerError(String::from("User with BanStatus::None despite ban_status.is_active == true"))), // should never happen
            },
            false => Ok(())
        }
    }

    pub fn check_can_publish_on_forum(&self, forum_name: &str) -> Result<(), AppError> {
        self.check_can_publish()?;
        match self.ban_status_by_forum_map.get(forum_name) {
            Some(ban_status) if ban_status.is_active() => match ban_status {
                BanStatus::Until(timestamp) => Err(AppError::ForumBanUntil(*timestamp)),
                BanStatus::Permanent => Err(AppError::PermanentForumBan),
                BanStatus::None => Err(AppError::InternalServerError(String::from("User with forum BanStatus::None despite ban_status.is_active == true"))), // should never happen
            },
            _ => Ok(())
        }
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use async_trait::async_trait;
    use axum_session::SessionPgPool;
    use sqlx::PgPool;

    use crate::errors::AppError;
    use crate::forum_management::UserBan;
    use crate::role::ssr::get_user_forum_role;
    use crate::role::UserForumRole;

    use super::*;

    pub type AuthSession = axum_session_auth::AuthSession<User, i64, SessionPgPool, PgPool>;

    #[derive(sqlx::FromRow, Clone, Debug, PartialEq)]
    pub struct SqlUser {
        pub user_id: i64,
        pub oidc_id: String,
        pub username: String,
        pub email: String,
        pub admin_role: AdminRole,
        pub timestamp: chrono::DateTime<chrono::Utc>,
        pub is_deleted: bool,
    }

    impl SqlUser {
        pub async fn get_from_oidc_id(oidc_id: &String, db_pool: &PgPool) -> Result<SqlUser, AppError> {
            let sql_user = sqlx::query_as!(
                SqlUser,
                "SELECT * FROM users WHERE oidc_id = $1",
                oidc_id
            )
                .fetch_one(db_pool)
                .await?;

            Ok(sql_user)
        }
        pub fn into_user(
            self,
            user_role_vec: Vec<UserForumRole>,
            user_ban_vec: Vec<UserBan>,
        ) -> User {
            let mut permission_by_forum_map: HashMap<String, PermissionLevel> = HashMap::new();
            for user_forum_role in user_role_vec {
                permission_by_forum_map.insert(
                    user_forum_role.forum_name.clone(),
                    user_forum_role.permission_level,
                );
            }
            let mut global_ban_status = BanStatus::None;
            let mut ban_status_by_forum_map: HashMap<String, BanStatus> = HashMap::new();
            let current_timestamp = chrono::offset::Utc::now();
            for user_ban in user_ban_vec {
                let (ban_status, is_valid) = match user_ban.until_timestamp {
                    Some(until_timestamp) => (
                        BanStatus::Until(until_timestamp),
                        until_timestamp > current_timestamp,
                    ),
                    None => (BanStatus::Permanent, true),
                };
                if is_valid {
                    match user_ban.forum_name {
                        Some(forum_name) => {
                            match ban_status_by_forum_map.get_mut(&forum_name) {
                                Some(current_ban_status) => {
                                    if ban_status > *current_ban_status {
                                        *current_ban_status = ban_status;
                                    }
                                },
                                None => _ = ban_status_by_forum_map.insert(forum_name, ban_status),
                            };
                        },
                        None => {
                            if ban_status > global_ban_status {
                                global_ban_status = ban_status;
                            }
                        }
                    };
                }
            }

            User {
                user_id: self.user_id,
                oidc_id: self.oidc_id,
                username: self.username,
                email: self.email,
                admin_role: self.admin_role,
                permission_by_forum_map,
                ban_status: global_ban_status,
                ban_status_by_forum_map,
                timestamp: self.timestamp,
                is_deleted: self.is_deleted,
            }
        }
    }

    impl User {
        pub async fn get(user_id: i64, db_pool: &PgPool) -> Option<Self> {
            match sqlx::query_as!(SqlUser, "SELECT * FROM users WHERE user_id = $1", user_id)
                .fetch_one(db_pool)
                .await
            {
                Ok(sql_user) => {
                    let user_forum_role_vec = load_user_forum_role_vec(sql_user.user_id, db_pool)
                        .await
                        .unwrap_or_default();
                    let user_ban_vec = load_user_ban_vec(sql_user.user_id, db_pool)
                        .await
                        .unwrap_or_default();
                    Some(sql_user.into_user(user_forum_role_vec, user_ban_vec))
                }
                Err(select_error) => {
                    log::debug!("User not found with error: {}", select_error);
                    None
                }
            }
        }

        pub async fn check_can_set_user_forum_role(
            &self,
            permission_level: PermissionLevel,
            user_id: i64,
            forum_name: &str,
            db_pool: &PgPool,
        ) -> Result<(), AppError> {
            match (self.admin_role, self.permission_by_forum_map.get(forum_name)) {
                (AdminRole::Admin, _) => Ok(()),
                (_, Some(own_level)) if *own_level >= PermissionLevel::Manage && *own_level > permission_level => {
                    match get_user_forum_role(user_id, forum_name, db_pool).await {
                        Err(AppError::NotFound) => Ok(()),
                        Ok(user_role) if *own_level > user_role.permission_level => Ok(()),
                        _ => Err(AppError::InsufficientPrivileges),
                    }
                },
                _ => Err(AppError::InsufficientPrivileges),
            }
        }
    }

    #[async_trait]
    impl Authentication<User, i64, PgPool> for User {
        async fn load_user(user_id: i64, pool: Option<&PgPool>) -> Result<User, anyhow::Error> {
            let pool = pool.ok_or(anyhow::anyhow!("Cannot get DB pool"))?;

            User::get(user_id, pool)
                .await
                .ok_or_else(|| anyhow::anyhow!("Cannot get user"))
        }

        fn is_authenticated(&self) -> bool {
            true
        }

        fn is_active(&self) -> bool {
            true
        }

        fn is_anonymous(&self) -> bool {
            false
        }
    }

    pub async fn create_user(
        oidc_id: &str,
        username: &str,
        email: &str,
        db_pool: &PgPool,
    ) -> Result<SqlUser, AppError> {
        log::debug!("Create new user {username}");
        let sql_user = sqlx::query_as!(
            SqlUser,
            "INSERT INTO users (oidc_id, username, email) VALUES ($1, $2, $3) RETURNING *",
            oidc_id,
            username,
            email,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(sql_user)
    }

    async fn load_user_forum_role_vec(
        user_id: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<UserForumRole>, AppError> {
        let user_forum_role_vec = sqlx::query_as!(
            UserForumRole,
            "SELECT * \
            FROM user_forum_roles \
            WHERE user_id = $1",
            user_id
        )
            .fetch_all(db_pool)
            .await?;
        log::trace!("User roles: {:?}", user_forum_role_vec);
        Ok(user_forum_role_vec)
    }

    async fn load_user_ban_vec(user_id: i64, db_pool: &PgPool) -> Result<Vec<UserBan>, AppError> {
        let user_ban_vec = sqlx::query_as!(
            UserBan,
            "SELECT * FROM user_bans WHERE user_id = $1 AND (until_timestamp > CURRENT_TIMESTAMP or until_timestamp IS NULL)",
            user_id
        )
            .fetch_all(db_pool)
            .await?;
        log::trace!("User bans: {:?}", user_ban_vec);
        Ok(user_ban_vec)
    }

    pub fn get_issuer_url() -> Result<oidc::IssuerUrl, AppError> {
        Ok(oidc::IssuerUrl::new(env::var(OIDC_ISSUER_URL_ENV)?)?)
    }

    pub fn get_client_id() -> Result<oidc::ClientId, AppError> {
        Ok(oidc::ClientId::new(env::var(AUTH_CLIENT_ID_ENV)?))
    }

    fn get_client_secret() -> Option<oidc::ClientSecret> {
        match env::var(AUTH_CLIENT_SECRET_ENV) {
            Ok(secret) => Some(oidc::ClientSecret::new(secret)),
            Err(_) => None,
        }
    }

    fn get_base_url() -> Result<String, AppError> {
        Ok(env::var(BASE_URL_ENV)?)
    }

    pub fn get_auth_redirect() -> Result<oidc::RedirectUrl, AppError> {
        Ok(oidc::RedirectUrl::new(String::from("http://") + get_base_url()?.as_str() + AUTH_CALLBACK_ROUTE)?)
    }

    pub fn get_logout_redirect() -> Result<oidc::PostLogoutRedirectUrl, AppError> {
        Ok(oidc::PostLogoutRedirectUrl::new(String::from("http://") + get_base_url()?.as_str())?)
    }

    pub async fn get_auth_client() -> Result<oidc::core::CoreClient, AppError> {
        let redirect_url = get_auth_redirect()?;
        let issuer_url = get_issuer_url()?;

        let provider_metadata =
            oidc::core::CoreProviderMetadata::discover_async(issuer_url.clone(), async_http_client)
                .await?;

        // Create an OpenID Connect client by specifying the client ID, client secret, authorization URL
        // and token URL.
        let client = oidc::core::CoreClient::from_provider_metadata(
            provider_metadata.clone(),
            get_client_id()?,
            get_client_secret(),
        )
        // Set the URL the user will be redirected to after the authorization process.
        .set_redirect_uri(redirect_url);

        Ok(client)
    }

    pub fn check_user() -> Result<User, AppError> {
        let auth_session = get_session()?;
        auth_session.current_user.ok_or(AppError::NotAuthenticated)
    }

    pub fn reload_user(user_id: i64) -> Result<(), AppError> {
        let auth_session = get_session()?;
        auth_session.cache_clear_user(user_id);
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use std::ops::Add;

        use chrono::Days;

        use super::*;

        #[test]
        fn test_sql_user_into_user() {
            let past_timestamp = chrono::DateTime::from_timestamp_nanos(0);
            let future_timestamp = chrono::offset::Utc::now().add(Days::new(1));
            let sql_user = SqlUser {
                user_id: 0,
                oidc_id: String::from("a"),
                username: String::from("b"),
                email: String::from("c"),
                admin_role: AdminRole::None,
                timestamp: chrono::DateTime::from_timestamp_nanos(0),
                is_deleted: false,
            };
            let user_forum_role_vec = vec![
                UserForumRole {
                    role_id: 0,
                    user_id: 0,
                    forum_id: 0,
                    forum_name: String::from("0"),
                    permission_level: PermissionLevel::Moderate,
                    grantor_id: 0,
                    timestamp: past_timestamp,
                },
                UserForumRole {
                    role_id: 0,
                    user_id: 0,
                    forum_id: 1,
                    forum_name: String::from("1"),
                    permission_level: PermissionLevel::Lead,
                    grantor_id: 0,
                    timestamp: past_timestamp,
                },
            ];
            let user_ban_vec = vec![
                UserBan {
                    ban_id: 0,
                    user_id: 0,
                    forum_id: None,
                    forum_name: None,
                    moderator_id: 0,
                    until_timestamp: Some(past_timestamp),
                    create_timestamp: Default::default(),
                },
                UserBan {
                    ban_id: 1,
                    user_id: 0,
                    forum_id: Some(0),
                    forum_name: Some(String::from("a")),
                    moderator_id: 0,
                    until_timestamp: Some(past_timestamp),
                    create_timestamp: Default::default(),
                },
                UserBan {
                    ban_id: 2,
                    user_id: 0,
                    forum_id: Some(1),
                    forum_name: Some(String::from("b")),
                    moderator_id: 0,
                    until_timestamp: Some(future_timestamp),
                    create_timestamp: Default::default(),
                },
                UserBan {
                    ban_id: 3,
                    user_id: 0,
                    forum_id: Some(2),
                    forum_name: Some(String::from("c")),
                    moderator_id: 0,
                    until_timestamp: None,
                    create_timestamp: Default::default(),
                },
            ];
            let user_1 = sql_user.clone().into_user(user_forum_role_vec.clone(), user_ban_vec);
            assert_eq!(user_1.user_id, 0);
            assert_eq!(user_1.oidc_id, "a");
            assert_eq!(user_1.username, "b");
            assert_eq!(user_1.email, "c");
            assert_eq!(user_1.admin_role, AdminRole::None);
            assert_eq!(user_1.timestamp, chrono::DateTime::from_timestamp_nanos(0));
            assert_eq!(user_1.is_deleted, false);
            assert_eq!(user_1.permission_by_forum_map[&String::from("0")], PermissionLevel::Moderate);
            assert_eq!(user_1.permission_by_forum_map[&String::from("1")], PermissionLevel::Lead);
            assert_eq!(user_1.ban_status, BanStatus::None);
            assert_eq!(user_1.ban_status_by_forum_map.get(&String::from("a")), None);
            assert_eq!(
                *user_1.ban_status_by_forum_map.get(&String::from("b")).expect("User should have ban for forum 'b'."),
                BanStatus::Until(future_timestamp)
            );
            assert_eq!(
                *user_1.ban_status_by_forum_map.get(&String::from("c")).expect("User should have ban for forum 'c'."),
                BanStatus::Permanent
            );

            let user_2_ban_vec = vec![UserBan {
                ban_id: 3,
                user_id: 0,
                forum_id: None,
                forum_name: None,
                moderator_id: 0,
                until_timestamp: Some(future_timestamp),
                create_timestamp: Default::default(),
            }];
            let user_2 = sql_user.into_user(user_forum_role_vec, user_2_ban_vec);
            assert_eq!(user_2.ban_status, BanStatus::Until(future_timestamp));
        }
    }
}

#[server]
pub async fn login(redirect_url: String) -> Result<User, ServerFnError> {
    let current_user = check_user();

    if current_user
        .as_ref()
        .is_ok_and(|user| user.is_authenticated())
    {
        log::info!(
            "Already logged in, current user is: {:?}",
            current_user.clone().unwrap()
        );
        return Ok(current_user.unwrap());
    }

    log::debug!("User not connected, redirect_url: {}", redirect_url);

    let client = ssr::get_auth_client().await?;

    // Generate the full authorization URL.
    let (auth_url, _csrf_token, nonce) = client
        .authorize_url(
            oidc::core::CoreAuthenticationFlow::AuthorizationCode,
            oidc::CsrfToken::new_random,
            oidc::Nonce::new_random,
        )
        // Set the desired scopes.
        //.add_scope(oidc::Scope::new("read".to_string()))
        //.add_scope(oidc::Scope::new("write".to_string()))
        .url();

    let auth_session = get_session()?;
    auth_session.session.set(NONCE_KEY, nonce);
    auth_session.session.set(REDIRECT_URL_KEY, redirect_url);

    // Redirect to the auth page
    leptos_axum::redirect(auth_url.as_ref());

    Ok(User::default())
}

#[server]
pub async fn authenticate_user(auth_code: String) -> Result<(), ServerFnError> {
    // Once the user has been redirected to the redirect URL, you'll have access to the
    // authorization code. For security reasons, your code should verify that the `state`
    // parameter returned by the server matches `csrf_state`.

    let auth_session = get_session()?;

    let nonce = oidc::Nonce::new(
        auth_session
            .session
            .get(NONCE_KEY)
            .unwrap_or(String::from("")),
    );
    let redirect_url = auth_session
        .session
        .get(REDIRECT_URL_KEY)
        .unwrap_or(String::from("/"));

    let client = ssr::get_auth_client().await?;

    // Now you can exchange it for an access token and ID token.
    let token_response = client
        .exchange_code(oidc::AuthorizationCode::new(auth_code))
        .request_async(async_http_client)
        .await?;

    // Extract the ID token claims after verifying its authenticity and nonce.
    let id_token = token_response
        .id_token()
        .ok_or(ServerFnError::new("Error getting id token."))?;
    let claims = id_token.claims(&client.id_token_verifier(), &nonce)?;

    // Verify the access token hash to ensure that the access token hasn't been substituted for another user's.
    if let Some(expected_access_token_hash) = claims.access_token_hash() {
        let actual_access_token_hash = oidc::AccessTokenHash::from_token(
            token_response.access_token(),
            &id_token.signing_alg()?,
        )?;
        if actual_access_token_hash != *expected_access_token_hash {
            return Err(ServerFnError::new("Invalid access token"));
        }
    }

    // The authenticated user's identity is now available. See the IdTokenClaims struct for a
    // complete listing of the available claims.
    log::debug!(
        "User {} with e-mail address {} has authenticated successfully",
        claims.subject().as_str(),
        claims
            .email()
            .map(|email| email.as_str())
            .unwrap_or("<not provided>"),
    );

    // If available, we can use the UserInfo endpoint to request additional information.

    // The user_info request uses the AccessToken returned in the token response. To parse custom
    // claims, use UserInfoClaims directly (with the desired type parameters) rather than using the
    // CoreUserInfoClaims type alias.
    let _userinfo: oidc::core::CoreUserInfoClaims = client
        .user_info(token_response.access_token().to_owned(), None)
        .map_err(|err| ServerFnError::new("No user info endpoint: ".to_owned() + &err.to_string()))?
        .request_async(async_http_client)
        .await
        .map_err(|err| {
            ServerFnError::new("Failed requesting user info: ".to_owned() + &err.to_string())
        })?;

    auth_session
        .session
        .set(OIDC_TOKENS_KEY, token_response.clone());

    let oidc_id = claims.subject().to_string();
    let db_pool = get_db_pool()?;

    let user = if let Ok(user) = SqlUser::get_from_oidc_id(&oidc_id, &db_pool).await {
        user
    } else {
        let username: String = claims.preferred_username().unwrap().to_string();
        let email: String = claims.email().unwrap().to_string();
        create_user(&oidc_id, &username, &email, &db_pool).await?
    };

    auth_session.login_user(user.user_id);

    leptos_axum::redirect(redirect_url.as_ref());

    Ok(())
}

#[server]
pub async fn get_user() -> Result<Option<User>, ServerFnError> {
    let auth_session = get_session()?;
    Ok(auth_session.current_user)
}

#[server]
pub async fn end_session(redirect_url: String) -> Result<(), ServerFnError> {
    log::debug!("Logout, redirect_url: {redirect_url}");

    let auth_session = get_session()?;
    log::debug!("Got session.");
    let token_response: oidc::core::CoreTokenResponse =
        auth_session
            .session
            .get(OIDC_TOKENS_KEY)
            .ok_or(ServerFnError::new("Not authenticated."))?;

    log::debug!("Got id token: {token_response:?}");

    let logout_provider_metadata =
        oidc::ProviderMetadataWithLogout::discover_async(ssr::get_issuer_url()?, async_http_client)
            .await?;

    let logout_endpoint: &Option<oidc::EndSessionUrl> = &logout_provider_metadata
        .additional_metadata()
        .end_session_endpoint;

    let logout_endpoint_url = match logout_endpoint {
        Some(url) => url.clone(),
        None => return Err(ServerFnError::new("Cannot get logout endpoint.")),
    };

    let logout_request = oidc::LogoutRequest::from(logout_endpoint_url)
        .set_client_id(ssr::get_client_id()?)
        .set_id_token_hint(token_response.id_token().unwrap())
        .set_post_logout_redirect_uri(oidc::PostLogoutRedirectUrl::new(redirect_url)?);

    leptos_axum::redirect(logout_request.http_get_url().to_string().as_str());

    auth_session.session.remove(OIDC_TOKENS_KEY);
    auth_session.logout_user();

    Ok(())
}

/// Component to guard a component requiring a login. If the user is logged in, a simple button with the given class and
/// children will be rendered. Otherwise, it will be replaced by a form/button with the same appearance redirecting to a
/// login screen.
#[component]
pub fn LoginGuardButton<F: Fn(&User) -> IV + 'static, IV: IntoView>(
    #[prop(default = "")] login_button_class: &'static str,
    #[prop(into)] login_button_content: ViewFn,
    #[prop(default = &get_current_path)] redirect_path_fn: &'static dyn Fn(RwSignal<String>),
    children: F,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let children = store_value(children);
    let login_button_content = store_value(login_button_content);

    view! {
        {
            move || state.user.with(|result| match result {
                Some(Ok(Some(user))) => children.with_value(|children| children(user)).into_view(),
                Some(_) => {
                    let login_button_view = login_button_content.get_value().run();
                    view! { <LoginButton class=login_button_class redirect_path_fn=redirect_path_fn>{login_button_view}</LoginButton> }.into_view()
                }
                _ => {
                    view! { <div class=login_button_class>{login_button_content.get_value().run()}</div> }.into_view()
                }
            })
        }
    }
}

#[component]
pub fn LoginButton(
    class: &'static str,
    #[prop(default = &get_current_path)] redirect_path_fn: &'static dyn Fn(RwSignal<String>),
    children: Children,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let redirect_path = create_rw_signal(String::default());

    view! {
        <form action=state.login_action.url() method="post" rel="external" class="flex items-center">
            <input type="text" name="redirect_url" class="hidden" value=redirect_path/>
            <button type="submit" class=class on:click=move |_| redirect_path_fn(redirect_path)>
                {children()}
            </button>
        </form>
    }
}

/// Auth callback component
#[component]
pub fn AuthCallback() -> impl IntoView {
    let query = use_query_map();
    let code = move || query.with_untracked(|query| query.get("code").unwrap().to_string());
    let auth_resource = create_blocking_resource(|| (), move |_| authenticate_user(code()));

    view! {
        <SuspenseUnpack
            resource=auth_resource
            let:_auth_result
        >
            {
                log::debug!("Authenticated successfully");
                View::default()
            }
        </SuspenseUnpack>
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Add;

    use chrono::Days;

    use super::*;

    fn get_user_permission_map() -> HashMap<String, PermissionLevel> {
        HashMap::from([
            (String::from("a"), PermissionLevel::None),
            (String::from("b"), PermissionLevel::Moderate),
            (String::from("c"), PermissionLevel::Ban),
            (String::from("d"), PermissionLevel::Manage),
            (String::from("e"), PermissionLevel::Lead),
        ])
    }

    #[test]
    fn test_ban_status_is_permanent() {
        let ban_status_none = BanStatus::None;
        let ban_status_until = BanStatus::Until(chrono::DateTime::from_timestamp_nanos(0));
        let ban_status_permanent = BanStatus::Permanent;
        assert_eq!(ban_status_none.is_permanent(), false);
        assert_eq!(ban_status_until.is_permanent(), false);
        assert_eq!(ban_status_permanent.is_permanent(), true);
    }

    #[test]
    fn test_ban_status_is_active() {
        let ban_status_none = BanStatus::None;
        let ban_status_until_past = BanStatus::Until(chrono::DateTime::from_timestamp_nanos(0));
        let ban_status_until_future = BanStatus::Until(chrono::offset::Utc::now().add(Days::new(1)));
        let ban_status_permanent = BanStatus::Permanent;
        assert_eq!(ban_status_none.is_active(), false);
        assert_eq!(ban_status_until_past.is_active(), false);
        assert_eq!(ban_status_until_future.is_active(), true);
        assert_eq!(ban_status_permanent.is_active(), true);
    }

    #[test]
    fn test_user_check_admin_role() {
        let mut user = User::default();
        assert_eq!(user.check_admin_role(AdminRole::None), Ok(()));
        assert_eq!(user.check_admin_role(AdminRole::Moderator), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_admin_role(AdminRole::Admin), Err(AppError::InsufficientPrivileges));
        user.admin_role = AdminRole::Moderator;
        assert_eq!(user.check_admin_role(AdminRole::None), Ok(()));
        assert_eq!(user.check_admin_role(AdminRole::Moderator), Ok(()));
        assert_eq!(user.check_admin_role(AdminRole::Admin), Err(AppError::InsufficientPrivileges));
        user.admin_role = AdminRole::Admin;
        assert_eq!(user.check_admin_role(AdminRole::None), Ok(()));
        assert_eq!(user.check_admin_role(AdminRole::Moderator), Ok(()));
        assert_eq!(user.check_admin_role(AdminRole::Admin), Ok(()));
    }

    #[test]
    fn test_user_check_can_moderate() {
        let mut user = User::default();
        user.permission_by_forum_map = get_user_permission_map();
        assert_eq!(
            user.check_can_moderate_forum("a"),
            Err(AppError::InsufficientPrivileges)
        );
        assert_eq!(user.check_can_moderate_forum("b"), Ok(()));
        assert_eq!(user.check_can_moderate_forum("c"), Ok(()));
        assert_eq!(user.check_can_moderate_forum("d"), Ok(()));
        assert_eq!(user.check_can_moderate_forum("e"), Ok(()));
        let mut admin = User::default();
        admin.admin_role = AdminRole::Moderator;
        assert_eq!(admin.check_can_moderate_forum("a"), Ok(()));
        admin.admin_role = AdminRole::Admin;
        assert_eq!(admin.check_can_moderate_forum("a"), Ok(()));
        admin.permission_by_forum_map = get_user_permission_map();
        assert_eq!(admin.check_can_ban_users("b"), Ok(()));
    }

    #[test]
    fn test_user_check_can_ban_users() {
        let mut user = User::default();
        user.permission_by_forum_map = get_user_permission_map();
        assert_eq!(user.check_can_ban_users("a"), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_can_ban_users("b"), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_can_ban_users("c"), Ok(()));
        assert_eq!(user.check_can_ban_users("d"), Ok(()));
        assert_eq!(user.check_can_ban_users("e"), Ok(()));
        let mut admin = User::default();
        admin.admin_role = AdminRole::Moderator;
        assert_eq!(admin.check_can_ban_users("a"), Ok(()));
        admin.admin_role = AdminRole::Admin;
        assert_eq!(admin.check_can_ban_users("a"), Ok(()));
        admin.permission_by_forum_map = get_user_permission_map();
        assert_eq!(admin.check_can_ban_users("c"), Ok(()));
    }

    #[test]
    fn test_user_check_can_manage_forum() {
        let mut user = User::default();
        user.permission_by_forum_map = get_user_permission_map();
        assert_eq!(user.check_can_manage_forum("a"), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_can_manage_forum("b"), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_can_manage_forum("c"), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_can_manage_forum("d"), Ok(()));
        assert_eq!(user.check_can_manage_forum("e"), Ok(()));
        let mut admin = User::default();
        admin.admin_role = AdminRole::Moderator;
        assert_eq!(admin.check_can_manage_forum("a"), Err(AppError::InsufficientPrivileges));
        admin.admin_role = AdminRole::Admin;
        assert_eq!(admin.check_can_manage_forum("a"), Ok(()));
        admin.permission_by_forum_map = get_user_permission_map();
        assert_eq!(admin.check_can_manage_forum("d"), Ok(()));
    }

    #[test]
    fn test_user_check_is_forum_leader() {
        let mut user = User::default();
        user.permission_by_forum_map = get_user_permission_map();
        assert_eq!(user.check_is_forum_leader("a"), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_is_forum_leader("b"), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_is_forum_leader("c"), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_is_forum_leader("d"), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_is_forum_leader("e"), Ok(()));
        let mut admin = User::default();
        admin.admin_role = AdminRole::Moderator;
        assert_eq!(admin.check_is_forum_leader("a"), Err(AppError::InsufficientPrivileges));
        admin.admin_role = AdminRole::Admin;
        assert_eq!(admin.check_is_forum_leader("a"), Err(AppError::InsufficientPrivileges));
        admin.permission_by_forum_map = get_user_permission_map();
        assert_eq!(admin.check_can_manage_forum("f"), Ok(()));
    }

    #[test]
    fn test_user_check_can_publish() {
        let past_timestamp = chrono::DateTime::from_timestamp_nanos(0);
        let future_timestamp = chrono::offset::Utc::now().add(Days::new(1));
        let mut user = User::default();
        assert_eq!(user.check_can_publish(), Ok(()));
        user.ban_status = BanStatus::Until(past_timestamp);
        assert_eq!(user.check_can_publish(), Ok(()));
        user.ban_status = BanStatus::Until(future_timestamp);
        assert_eq!(user.check_can_publish(), Err(AppError::GlobalBanUntil(future_timestamp)));
        user.ban_status = BanStatus::Permanent;
        assert_eq!(user.check_can_publish(), Err(AppError::PermanentGlobalBan));
    }

    #[test]
    fn test_user_check_can_publish_on_forum() {
        let past_timestamp = chrono::DateTime::from_timestamp_nanos(0);
        let future_timestamp = chrono::offset::Utc::now().add(Days::new(1));
        let mut user = User::default();
        user.ban_status_by_forum_map = HashMap::from([
            (String::from("a"), BanStatus::None),
            (String::from("b"), BanStatus::Until(past_timestamp)),
            (String::from("c"), BanStatus::Until(future_timestamp)),
            (String::from("d"), BanStatus::Permanent),
        ]);
        assert_eq!(user.check_can_publish_on_forum("a"), Ok(()));
        assert_eq!(user.check_can_publish_on_forum("b"), Ok(()));
        assert_eq!(user.check_can_publish_on_forum("c"), Err(AppError::ForumBanUntil(future_timestamp)));
        assert_eq!(user.check_can_publish_on_forum("d"), Err(AppError::PermanentForumBan));
        user.ban_status = BanStatus::Until(past_timestamp);
        assert_eq!(user.check_can_publish_on_forum("a"), Ok(()));
        assert_eq!(user.check_can_publish_on_forum("b"), Ok(()));
        assert_eq!(user.check_can_publish_on_forum("c"), Err(AppError::ForumBanUntil(future_timestamp)));
        assert_eq!(user.check_can_publish_on_forum("d"), Err(AppError::PermanentForumBan));
        user.ban_status = BanStatus::Until(future_timestamp);
        assert_eq!(user.check_can_publish_on_forum("a"), Err(AppError::GlobalBanUntil(future_timestamp)));
        assert_eq!(user.check_can_publish_on_forum("b"), Err(AppError::GlobalBanUntil(future_timestamp)));
        assert_eq!(user.check_can_publish_on_forum("c"), Err(AppError::GlobalBanUntil(future_timestamp)));
        assert_eq!(user.check_can_publish_on_forum("d"), Err(AppError::GlobalBanUntil(future_timestamp)));
        user.ban_status = BanStatus::Permanent;
        assert_eq!(user.check_can_publish_on_forum("a"), Err(AppError::PermanentGlobalBan));
        assert_eq!(user.check_can_publish_on_forum("b"), Err(AppError::PermanentGlobalBan));
        assert_eq!(user.check_can_publish_on_forum("c"), Err(AppError::PermanentGlobalBan));
        assert_eq!(user.check_can_publish_on_forum("d"), Err(AppError::PermanentGlobalBan));
    }
}
