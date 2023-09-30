use cfg_if::cfg_if;
use std::env;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

use crate::app::GlobalState;
use crate::icons::*;

pub const BASE_URL_ENV : &str = "LEPTOS_SITE_ADDR";
pub const AUTH_CLIENT_ID_ENV : &str = "AUTH_CLIENT_ID";
pub const AUTH_CLIENT_SECRET_ENV : &str = "AUTH_CLIENT_SECRET";
pub const AUTH_CALLBACK_ROUTE : &str = "/authback";
pub const PKCE_KEY : &str = "pkce";
pub const NONCE_KEY : &str = "nonce";
pub const OIDC_TOKENS_KEY : &str = "oidc_token";
pub const USER_KEY : &str = "user";
pub const REDIRECT_URL_KEY : &str = "redirect";

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use openidconnect as oidc;
        use openidconnect::reqwest::*;
        use openidconnect::{OAuth2TokenResponse, TokenResponse};
        use sqlx::PgPool;
        use axum_session::SessionPgPool;

        pub type Session = axum_session::Session<SessionPgPool>;

        pub fn get_db_pool(cx: Scope) -> Result<PgPool, ServerFnError> {
            use_context::<PgPool>(cx).ok_or_else(|| ServerFnError::ServerError("Pool missing.".into()))
        }

        pub fn get_session(cx: Scope) -> Result<Session, ServerFnError> {
            use_context::<Session>(cx).ok_or_else(|| ServerFnError::ServerError("Session missing.".into()))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub anonymous: bool,
    pub username: String,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: String::default(),
            anonymous: true,
            username: String::default(),
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
pub async fn is_user_authenticated(cx: Scope) -> bool {
    match get_user(cx).await {
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

#[server(Login, "/api")]
pub async fn login(cx: Scope, redirect_url: String) -> Result<User, ServerFnError> {

    let current_user = get_user(cx).await;

    if current_user.is_ok() {
        log!("Already logged in, current user is: {:?}", current_user.clone().unwrap());
        return current_user;
    }

    log!("User not connected, redirect_url: {}", redirect_url);

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

    let session = get_session(cx)?;
    session.set(NONCE_KEY, nonce);
    session.set(REDIRECT_URL_KEY, redirect_url);

    // Redirect to the auth page
    leptos_axum::redirect(cx, auth_url.as_ref());

    Ok(User::default())
}

#[server(AuthenticateUser, "/api")]
pub async fn authenticate_user(cx: Scope, auth_code: String) -> Result<(User, String), ServerFnError> {
    // Once the user has been redirected to the redirect URL, you'll have access to the
    // authorization code. For security reasons, your code should verify that the `state`
    // parameter returned by the server matches `csrf_state`.

    let session = get_session(cx)?;

    let nonce = oidc::Nonce::new(session.get(NONCE_KEY).unwrap_or(String::from("")));
    let redirect_url = session.get(REDIRECT_URL_KEY).unwrap_or(String::from("/"));

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

    session.set(OIDC_TOKENS_KEY, token_response.clone());

    leptos_axum::redirect(cx, redirect_url.as_ref());

    let user = User {
        id: claims.subject().to_string(),
        anonymous: false,
        username: claims.preferred_username().unwrap().to_string(),
    };

    session.set(USER_KEY, user.clone());

    Ok((user, redirect_url))
}

#[server(GetUser, "/api")]
pub async fn get_user(cx: Scope) -> Result<User, ServerFnError> {
    let session = get_session(cx)?;
    let user: User = session.get(USER_KEY).ok_or(ServerFnError::ServerError(String::from("Not authenticated.")))?;

    Ok(user)
}

#[server(EndSession, "/api")]
pub async fn end_session(cx: Scope, redirect_url: String) -> Result<(), ServerFnError> {
    println!("Logout, redirect_url: {redirect_url}");

    let session = get_session(cx)?;
    let token_response: oidc::core::CoreTokenResponse = session.get(OIDC_TOKENS_KEY).ok_or(ServerFnError::ServerError(String::from("Not authenticated.")))?;

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

    leptos_axum::redirect(cx, logout_request.http_get_url().to_string().as_str());

    session.remove(OIDC_TOKENS_KEY);

    Ok(())
}

pub fn get_user_resource(cx: Scope) -> Resource<(RwSignal<usize>, RwSignal<usize>), Result<User, ServerFnError>> {
    let state = expect_context::<GlobalState>(cx);
    create_blocking_resource(
        cx,
        move || {
            (
                state.login_action.version(),
                state.logout_action.version(),
            )
        },
        move |_| { get_user(cx) },
    )
}

/// Auth callback component
#[component]
pub fn AuthCallback(
    cx: Scope) -> impl IntoView {
    use crate::app::*;
    let _state = expect_context::<GlobalState>(cx);
    let query = use_query_map(cx);
    let code = move || query().get("code").unwrap().to_owned();
    let auth_resource = create_blocking_resource(cx, || (), move |_| authenticate_user(cx, code()));

    view! { cx,
        <Suspense fallback=move || (view! {cx, <LoadingIcon/>})>
            {
                move || {
                    auth_resource.read(cx).map(|userResult| {
                            if let Ok((user, redirect_url)) = userResult {
                                log!("Store authenticated as {}", user.username);
                                log!("Redirect to {}", redirect_url);
                                view! {cx, <Redirect path=redirect_url/>}.into_view(cx)
                            }
                            else {
                                view! {cx, <div>"Authentication failed."</div>}.into_view(cx)
                            }
                        }
                    )
                }
            }
        </Suspense>
    }
}

