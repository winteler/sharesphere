use cfg_if::cfg_if;
use std::env;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

use crate::app::{GlobalState};
use crate::icons::*;
use crate::navigation_bar::get_current_path_closure;

pub const BASE_URL_ENV : &str = "LEPTOS_SITE_ADDR";
pub const AUTH_CLIENT_ID_ENV : &str = "AUTH_CLIENT_ID";
pub const AUTH_CLIENT_SECRET_ENV : &str = "AUTH_CLIENT_SECRET";
pub const AUTH_CALLBACK_ROUTE : &str = "/authback";
pub const PKCE_KEY : &str = "pkce";
pub const NONCE_KEY : &str = "nonce";
pub const OIDC_TOKENS_KEY : &str = "oidc_token";
pub const OIDC_USERNAME_KEY : &str = "oidc_username";
pub const REDIRECT_URL_KEY : &str = "redirect";

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub anonymous: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: -1,
            username: String::default(),
            anonymous: true,
            timestamp: chrono::DateTime::default(),
        }
    }
}

#[derive(Clone, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct OidcUserInfo {
    pub oidc_id: String,
    pub username: String,
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use async_trait::async_trait;
        use axum_session_auth::{SessionPgPool, Authentication};
        use openidconnect as oidc;
        use openidconnect::reqwest::*;
        use openidconnect::{OAuth2TokenResponse, TokenResponse};
        use sqlx::PgPool;

        pub type AuthSession = axum_session_auth::AuthSession<User, OidcUserInfo, SessionPgPool, PgPool>;

        #[derive(sqlx::FromRow, Clone)]
        pub struct SqlUser {
            pub id: i64,
            pub oidc_id: String,
            pub username: String,
            pub timestamp: chrono::DateTime<chrono::Utc>,
        }

        pub fn get_db_pool() -> Result<PgPool, ServerFnError> {
            use_context::<PgPool>().ok_or_else(|| ServerFnError::ServerError("Pool missing.".into()))
        }

        pub fn get_session() -> Result<AuthSession, ServerFnError> {
            use_context::<AuthSession>().ok_or_else(|| ServerFnError::ServerError("Auth session missing.".into()))
        }

        impl SqlUser {
            pub fn into_user(self) -> User {
                User {
                    id: self.id,
                    anonymous: false,
                    username: self.username,
                    timestamp: self.timestamp,
                }
            }
        }

        impl User {
            #[cfg(feature = "ssr")]
            pub async fn get(oidc_info: OidcUserInfo, pool: &PgPool) -> Option<Self> {
                log::info!("Try to get user from the DB");
                match sqlx::query_as!(
                    SqlUser,
                    "SELECT * FROM users WHERE oidc_id = $1",
                    oidc_info.oidc_id.clone()
                )
                    .fetch_one(pool)
                    .await {
                    Ok(sql_user) => Some(sql_user.into_user()),
                    Err(select_error) => {
                        log::info!("User not found with error: {}", select_error);
                        if let sqlx::Error::RowNotFound = select_error {
                            log::info!("Try to insert new user");
                            match sqlx::query_as!(
                                SqlUser,
                                "INSERT INTO users (oidc_id, username) VALUES ($1, $2) RETURNING *",
                                oidc_info.oidc_id,
                                oidc_info.username
                            )
                                .fetch_one(pool)
                                .await
                            {
                                Ok(sql_user) => Some(sql_user.into_user()),
                                Err(insert_error) => {
                                    log::error!("Error while storing new user: {}", insert_error);
                                    None
                                },
                            }
                        }
                        else {
                            log::error!("Could not get user {}", select_error);
                            None
                        }
                    }
                }
            }
        }

        #[async_trait]
        impl Authentication<User, OidcUserInfo, PgPool> for User {
            async fn load_user(id: OidcUserInfo, pool: Option<&PgPool>) -> Result<User, anyhow::Error> {
                let pool = pool.ok_or(anyhow::anyhow!("Cannot get DB pool"))?;

                User::get(id, pool)
                    .await
                    .ok_or_else(|| anyhow::anyhow!("Cannot get user"))
            }

            fn is_authenticated(&self) -> bool {
                !self.anonymous
            }

            fn is_active(&self) -> bool {
                !self.anonymous
            }

            fn is_anonymous(&self) -> bool {
                self.anonymous
            }
        }
    }
}

#[cfg(feature = "ssr")]
pub fn get_issuer_url() -> Result<oidc::IssuerUrl, ServerFnError> {
    Ok(oidc::IssuerUrl::new("http://127.0.0.1:8080/realms/project".to_string()).expect("Invalid issuer URL"))
}

#[cfg(feature = "ssr")]
pub fn get_client_id() -> Result<oidc::ClientId, ServerFnError> {
    Ok(oidc::ClientId::new(env::var(AUTH_CLIENT_ID_ENV)?))
}

#[cfg(feature = "ssr")]
pub fn get_client_secret() -> Option<oidc::ClientSecret> {
    match env::var(AUTH_CLIENT_SECRET_ENV) {
        Ok(secret) => Some(oidc::ClientSecret::new(secret)),
        Err(_) => None
    }
}

#[cfg(feature = "ssr")]
pub fn get_base_url() -> Result<String, ServerFnError> {
    Ok(env::var(BASE_URL_ENV)?)
}

#[cfg(feature = "ssr")]
pub fn get_auth_redirect() -> Result<oidc::RedirectUrl, ServerFnError> {
    Ok(oidc::RedirectUrl::new(String::from("http://") + get_base_url()?.as_str() + AUTH_CALLBACK_ROUTE)?)
}

#[cfg(feature = "ssr")]
pub fn get_logout_redirect() -> Result<oidc::PostLogoutRedirectUrl, ServerFnError> {
    Ok(oidc::PostLogoutRedirectUrl::new(String::from("http://") + get_base_url()?.as_str())?)
}

#[cfg(feature = "ssr")]
pub async fn is_user_authenticated() -> bool {
    match get_user().await {
        Ok(user) => !user.anonymous,
        Err(_) => false
    }
}

#[cfg(feature = "ssr")]
pub async fn get_auth_client() -> Result<oidc::core::CoreClient, ServerFnError> {
    let redirect_url = get_auth_redirect()?;
    let issuer_url = get_issuer_url()?;

    let provider_metadata = oidc::core::CoreProviderMetadata::discover_async(
        issuer_url.clone(),
        async_http_client
    ).await?;

    // Create an OpenID Connect client by specifying the client ID, client secret, authorization URL
    // and token URL.
    let client =
        oidc::core::CoreClient::from_provider_metadata(
            provider_metadata.clone(),
            get_client_id()?,
            get_client_secret(),
        )
        // Set the URL the user will be redirected to after the authorization process.
        .set_redirect_uri(redirect_url);

    Ok(client)
}

#[server]
pub async fn login( redirect_url: String) -> Result<User, ServerFnError> {

    let current_user = get_user().await;

    if current_user.as_ref().is_ok_and(|user| user.is_authenticated()) {
        log::info!("Already logged in, current user is: {:?}", current_user.clone().unwrap());
        return current_user;
    }

    log::info!("User not connected, redirect_url: {}", redirect_url);

    let client = get_auth_client().await?;

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
    leptos_axum::redirect( auth_url.as_ref());

    Ok(User::default())
}

#[server]
pub async fn authenticate_user( auth_code: String) -> Result<(User, String), ServerFnError> {
    // Once the user has been redirected to the redirect URL, you'll have access to the
    // authorization code. For security reasons, your code should verify that the `state`
    // parameter returned by the server matches `csrf_state`.

    let auth_session = get_session()?;

    let nonce = oidc::Nonce::new(auth_session.session.get(NONCE_KEY).unwrap_or(String::from("")));
    let redirect_url = auth_session.session.get(REDIRECT_URL_KEY).unwrap_or(String::from("/"));

    println!("auth_code = {}", auth_code);
    println!("nonce = {:?}", nonce);

    let client = get_auth_client().await?;

    // Now you can exchange it for an access token and ID token.
    let token_response =
        client
            .exchange_code(oidc::AuthorizationCode::new(auth_code))
            .request_async(async_http_client).await?;

    // Extract the ID token claims after verifying its authenticity and nonce.
    let id_token = token_response.id_token().ok_or(ServerFnError::ServerError("Error getting id token.".to_owned()))?;
    let claims = id_token.claims(&client.id_token_verifier(), &nonce)?;

    // Verify the access token hash to ensure that the access token hasn't been substituted for another user's.
    if let Some(expected_access_token_hash) = claims.access_token_hash() {
        let actual_access_token_hash = oidc::AccessTokenHash::from_token(
            token_response.access_token(),
            &id_token.signing_alg()?
        )?;
        if actual_access_token_hash != *expected_access_token_hash {
            return Err(ServerFnError::ServerError("Invalid access token".to_owned()));
        }
    }

    // The authenticated user's identity is now available. See the IdTokenClaims struct for a
    // complete listing of the available claims.
    println!(
        "User {} with e-mail address {} has authenticated successfully",
        claims.subject().as_str(),
        claims.email().map(|email| email.as_str()).unwrap_or("<not provided>"),
    );

    // If available, we can use the UserInfo endpoint to request additional information.

    // The user_info request uses the AccessToken returned in the token response. To parse custom
    // claims, use UserInfoClaims directly (with the desired type parameters) rather than using the
    // CoreUserInfoClaims type alias.
    let _userinfo: oidc::core::CoreUserInfoClaims = client
        .user_info(token_response.access_token().to_owned(), None)
        .map_err(|err| ServerFnError::ServerError("No user info endpoint: ".to_owned() + &err.to_string()))?
        .request_async(async_http_client).await
        .map_err(|err| ServerFnError::ServerError("Failed requesting user info: ".to_owned() + &err.to_string()))?;

    auth_session.session.set(OIDC_TOKENS_KEY, token_response.clone());

    let oidc_user_info = OidcUserInfo {
        oidc_id: claims.subject().to_string(),
        username: claims.preferred_username().unwrap().to_string(),
    };

    auth_session.login_user(oidc_user_info);

    leptos_axum::redirect( redirect_url.as_ref());

    Ok((auth_session.current_user.unwrap_or_default(), redirect_url))
}

#[server]
pub async fn get_user() -> Result<User, ServerFnError> {
    let auth_session = get_session()?;
    match auth_session.current_user {
        Some(user) => {
            if !user.anonymous {
                Ok(user)
            }
            else {
                Err(ServerFnError::ServerError(String::from("Anonymous user.")))
            }
        },
        None => Err(ServerFnError::ServerError(String::from("Not authenticated."))),
    }
}

#[server]
pub async fn end_session( redirect_url: String) -> Result<(), ServerFnError> {
    log::info!("Logout, redirect_url: {redirect_url}");

    let auth_session = get_session()?;
    log::debug!("Got session.");
    let token_response: oidc::core::CoreTokenResponse = auth_session.session.get(OIDC_TOKENS_KEY).ok_or(ServerFnError::ServerError(String::from("Not authenticated.")))?;

    log::debug!("Got id token: {:?}", token_response);

    let logout_provider_metadata = oidc::ProviderMetadataWithLogout::discover_async(
        get_issuer_url()?,
        async_http_client
    ).await?;

    let logout_endpoint: &Option<oidc::EndSessionUrl> = &logout_provider_metadata
        .additional_metadata()
        .end_session_endpoint;

    let logout_endpoint_url = match logout_endpoint {
        Some(url) => url.clone(),
        None => return Err(ServerFnError::ServerError(String::from("Cannot get logout endpoint.")))
    };

    let logout_request = oidc::LogoutRequest::from(logout_endpoint_url)
        .set_client_id(get_client_id()?)
        .set_id_token_hint(token_response.id_token().unwrap())
        .set_post_logout_redirect_uri(oidc::PostLogoutRedirectUrl::new(redirect_url)?);

    leptos_axum::redirect(logout_request.http_get_url().to_string().as_str());

    auth_session.session.remove(OIDC_TOKENS_KEY);
    auth_session.logout_user();

    Ok(())
}

/// Component to guard a component requiring a login. If the user is logged in, a simple button with the given class and
/// children will be rendered. Otherwise, it will be replace by a form/button with the same appearance redirecting to a
/// login screen.
#[component]
pub fn LoginGuardButton(
    login_button_class: &'static str,
    #[prop(into)]
    login_button_content: ViewFn,
    children: Children,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();

    let login_button_content = create_memo({
        move |_| {
            login_button_content.run()
        }
    });

    state.user.with(|result| {
        match result {
            Some(Ok(_user)) => {
                children().into_view()
            },
            Some(Err(e)) => {
                log::info!("Error while getting user: {}", e);
                view! { <LoginButton class=login_button_class>{login_button_content()}</LoginButton> }.into_view()
            },
            None => {
                log::trace!("Resource not loaded yet.");
                view! { <button class=login_button_class/> }.into_view()
            }
        }
    })
}

#[component]
pub fn LoginGuardButtonWithUser<F>(
    login_button_class: &'static str,
    #[prop(into)]
    login_button_content: ViewFn,
    children_content: F,
) -> impl IntoView
    where
        F: Fn(&User) -> View + 'static,
{
    let state = expect_context::<GlobalState>();

    let login_button_memo = create_memo({
        move |_| {
            login_button_content.run()
        }
    });

    let content = create_memo({
        move |_| {
            state.user.with(|result| {
                match result {
                    Some(Ok(user)) => {
                        children_content(&user)
                    },
                    Some(Err(e)) => {
                        log::info!("Error while getting user: {}", e);
                        view! { <LoginButton class=login_button_class>{login_button_memo()}</LoginButton> }.into_view()
                    },
                    None => {
                        log::trace!("Resource not loaded yet.");
                        view! { <button class=login_button_class/> }.into_view()
                    }
                }
            })
        }
    });

    content.get()
    /*state.user.with(|result| {
        match result {
            Some(Ok(user)) => {
                children_content()
            },
            Some(Err(e)) => {
                log::info!("Error while getting user: {}", e);
                view! { <LoginButton class=login_button_class>{login_button_content()}</LoginButton> }.into_view()
            },
            None => {
                log::trace!("Resource not loaded yet.");
                view! { <button class=login_button_class/> }.into_view()
            }
        }
    })*/
}

#[component]
pub fn LoginButton(
    class: &'static str,
    children: Children,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let current_path = create_rw_signal( String::default());
    let get_current_path = get_current_path_closure(current_path);

    view! {
        <form action=state.login_action.url() method="post" rel="external">
            <input type="text" name="redirect_url" class="hidden" value=current_path/>
            <button type="submit" class=class on:click=get_current_path>
                {children()}
            </button>
        </form>
    }
}

/// Auth callback component
#[component]
pub fn AuthCallback(
    ) -> impl IntoView {
    use crate::app::*;
    let _state = expect_context::<GlobalState>();
    let query = use_query_map();
    let code = move || query.with_untracked(|query| query.get("code").unwrap().to_string());
    let auth = create_blocking_resource( || (), move |_| authenticate_user( code()));

    view! {
        <Suspense fallback=move || (view! { <LoadingIcon/>})>
            {
                move || {
                    auth.with(|result| {
                        if let Some(Ok((user, redirect_url))) = result {
                            log::info!("Store authenticated as {}", user.username);
                            log::info!("Redirect to {}", redirect_url);
                            view! { <Redirect path=redirect_url.clone()/>}.into_view()
                        }
                        else {
                            view! { <div>"Authentication failed."</div>}.into_view()
                        }
                    })
                }
            }
        </Suspense>
    }
}

