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
use crate::app::ssr::get_session;
#[cfg(feature = "ssr")]
use crate::auth::ssr::check_user;
use crate::navigation_bar::get_current_path;
use crate::role::{AdminRole, UserForumRole, UserRole};
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



#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub user_id: i64,
    pub oidc_id: String,
    pub username: String,
    pub email: String,
    pub admin_role: AdminRole,
    pub user_role_by_forum_map: HashMap<String, UserForumRole>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub is_deleted: bool,
}

impl Default for User {
    fn default() -> Self {
        Self {
            user_id: -1,
            oidc_id: String::default(),
            username: String::default(),
            email: String::default(),
            admin_role: AdminRole::None,
            user_role_by_forum_map: HashMap::new(),
            timestamp: chrono::DateTime::default(),
            is_deleted: false,
        }
    }
}

#[derive(Clone, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct OidcUserInfo {
    pub oidc_id: String,
    pub username: String,
    pub email: String,
}

impl std::fmt::Display for OidcUserInfo {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            fmt,
            "OidcUserInfo {{{}, {}, {}}}",
            self.oidc_id, self.username, self.email
        )
    }
}

impl User {

    pub fn is_global_moderator(&self) -> bool {
        self.admin_role > AdminRole::Moderator
    }

    pub fn is_admin(&self) -> bool {
        self.admin_role == AdminRole::Admin
    }
    pub fn is_forum_moderator(&self, forum_name: &String) -> bool {
        self.admin_role > AdminRole::Moderator ||
            self.user_role_by_forum_map.get(forum_name).is_some_and(|user_forum_role| user_forum_role.user_role > UserRole::Moderator)
    }

    pub fn is_forum_leader(&self, forum_name: String) -> bool {
        self.user_role_by_forum_map.get(&forum_name).is_some_and(|user_forum_role| user_forum_role.user_role == UserRole::Leader)
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use async_trait::async_trait;
    use axum_session::SessionPgPool;
    use sqlx::PgPool;

    use crate::errors::AppError;

    use super::*;

    pub type AuthSession =
        axum_session_auth::AuthSession<User, OidcUserInfo, SessionPgPool, PgPool>;

    #[derive(sqlx::FromRow, Clone)]
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
        pub fn into_user(self, user_role_vec: Vec<UserForumRole>) -> User {
            let mut user_role_by_forum_map = HashMap::new();
            for user_forum_role in user_role_vec {
                user_role_by_forum_map.insert(user_forum_role.forum_name.clone(), user_forum_role);
            }
            User {
                user_id: self.user_id,
                oidc_id: self.oidc_id,
                username: self.username,
                email: self.email,
                admin_role: self.admin_role,
                user_role_by_forum_map,
                timestamp: self.timestamp,
                is_deleted: self.is_deleted,
            }
        }
    }

    impl User {
        pub async fn get(oidc_info: OidcUserInfo, db_pool: &PgPool) -> Option<Self> {
            match sqlx::query_as!(
                SqlUser,
                "SELECT * FROM users WHERE oidc_id = $1",
                oidc_info.oidc_id.clone()
            )
            .fetch_one(db_pool)
            .await
            {
                Ok(sql_user) => {
                    let user_forum_role_vec = load_user_forum_role_vec(sql_user.user_id, db_pool).await.unwrap_or_default();
                    Some(sql_user.into_user(user_forum_role_vec))
                },
                Err(select_error) => {
                    log::debug!("User not found with error: {}", select_error);
                    if let sqlx::Error::RowNotFound = select_error {
                        create_user(oidc_info, db_pool).await
                    } else {
                        log::error!("Could not get user {}", select_error);
                        None
                    }
                }
            }
        }
    }

    #[async_trait]
    impl Authentication<User, OidcUserInfo, PgPool> for User {
        async fn load_user(
            oidc_id: OidcUserInfo,
            pool: Option<&PgPool>,
        ) -> Result<User, anyhow::Error> {
            let pool = pool.ok_or(anyhow::anyhow!("Cannot get DB pool"))?;

            User::get(oidc_id, pool)
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

    pub async fn create_user(oidc_info: OidcUserInfo, db_pool: &PgPool) -> Option<User> {
        log::debug!("Try to insert new user");
        match sqlx::query_as!(
            SqlUser,
            "INSERT INTO users (oidc_id, username, email) VALUES ($1, $2, $3) RETURNING *",
            oidc_info.oidc_id,
            oidc_info.username,
            oidc_info.email,
        )
        .fetch_one(db_pool)
        .await
        {
            Ok(sql_user) => Some(sql_user.into_user(Vec::new())),
            Err(insert_error) => {
                log::error!("Error while creating user: {}", insert_error);
                None
            }
        }
    }

    pub async fn load_user_forum_role_vec(user_id: i64, db_pool: &PgPool) -> Result<Vec<UserForumRole>, AppError> {
        let user_forum_role_vec = sqlx::query_as!(
            UserForumRole,
            "SELECT * FROM user_forum_roles WHERE user_id = $1",
            user_id
        )
            .fetch_all(db_pool)
            .await?;
        log::trace!("User roles: {:?}", user_forum_role_vec);
        Ok(user_forum_role_vec)
    }

    pub fn get_issuer_url() -> Result<oidc::IssuerUrl, AppError> {
        Ok(oidc::IssuerUrl::new(env::var(OIDC_ISSUER_URL_ENV)?)?)
    }

    pub fn get_client_id() -> Result<oidc::ClientId, AppError> {
        Ok(oidc::ClientId::new(env::var(AUTH_CLIENT_ID_ENV)?))
    }

    pub fn get_client_secret() -> Option<oidc::ClientSecret> {
        match env::var(AUTH_CLIENT_SECRET_ENV) {
            Ok(secret) => Some(oidc::ClientSecret::new(secret)),
            Err(_) => None,
        }
    }

    pub fn get_base_url() -> Result<String, AppError> {
        Ok(env::var(BASE_URL_ENV)?)
    }

    pub fn get_auth_redirect() -> Result<oidc::RedirectUrl, AppError> {
        Ok(oidc::RedirectUrl::new(
            String::from("http://") + get_base_url()?.as_str() + AUTH_CALLBACK_ROUTE,
        )?)
    }

    pub fn get_logout_redirect() -> Result<oidc::PostLogoutRedirectUrl, AppError> {
        Ok(oidc::PostLogoutRedirectUrl::new(
            String::from("http://") + get_base_url()?.as_str(),
        )?)
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

    pub fn reload_user() -> Result<Option<User>, AppError> {
        let auth_session = get_session()?;
        if let Some(current_user) = auth_session.current_user.clone() {
            auth_session.logout_user();
            let oidc_user_info = OidcUserInfo {
                oidc_id: current_user.oidc_id.clone(),
                username: current_user.username.clone(),
                email: current_user.email.clone(),
            };
            auth_session.cache_clear_user(oidc_user_info.clone());
            auth_session.login_user(oidc_user_info);
            log::trace!("Reloaded user: {:?}", auth_session.current_user);
        }
        Ok(auth_session.current_user)
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

    let oidc_user_info = OidcUserInfo {
        oidc_id: claims.subject().to_string(),
        username: claims.preferred_username().unwrap().to_string(),
        email: claims.email().unwrap().to_string(),
    };

    auth_session.login_user(oidc_user_info);

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
