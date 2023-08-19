use cfg_if::cfg_if;
use std::env;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

use crate::app::{GlobalState};

pub const BASE_URL_ENV : &str = "LEPTOS_SITE_ADDR";
pub const AUTH_CLIENT_ID_ENV : &str = "AUTH_CLIENT_ID";
pub const AUTH_CLIENT_SECRET_ENV : &str = "AUTH_CLIENT_SECRET";
pub const AUTH_CALLBACK_ROUTE : &str = "/authback";
pub const PKCE_KEY : &str = "pkce";
pub const NONCE_KEY : &str = "nonce";
pub const ID_TOKEN_KEY : &str = "id_token";
pub const ACCESS_TOKEN_KEY : &str = "access_token";
pub const REFRESH_TOKEN_KEY : &str = "refresh_token";

cfg_if! {
if #[cfg(feature = "ssr")] {

    use sqlx::{PgPool};
    use axum_session_auth::{SessionPgPool, Authentication};

    use openidconnect as oidc;
    use openidconnect::reqwest::*;
    // Use OpenID Connect Discovery to fetch the provider metadata.
    use openidconnect::{OAuth2TokenResponse, TokenResponse};

    pub type AuthSession = axum_session_auth::AuthSession<User, String, SessionPgPool, PgPool>;
}}

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

cfg_if! {
if #[cfg(feature = "ssr")] {
    use async_trait::async_trait;

    pub fn get_db_pool(cx: Scope) -> Result<PgPool, ServerFnError> {
       use_context::<PgPool>(cx)
            .ok_or_else(|| ServerFnError::ServerError("Pool missing.".into()))
    }

    pub fn get_session(cx: Scope) -> Result<AuthSession, ServerFnError> {
        use_context::<AuthSession>(cx)
            .ok_or_else(|| ServerFnError::ServerError("Auth session missing.".into()))
    }

    #[async_trait]
    impl Authentication<User, String, PgPool> for User {
        async fn load_user(_id: String, _pool: Option<&PgPool>) -> Result<User, anyhow::Error> {
            Ok(User::default())
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
}}

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

#[server(StartAuth, "/api")]
pub async fn start_auth(cx: Scope) -> Result<(), ServerFnError> {

    let client = get_auth_client().await?;

    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = oidc::PkceCodeChallenge::new_random_sha256();

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
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();

    let session = get_session(cx)?;
    session.session.set(NONCE_KEY, nonce);
    session.session.set(PKCE_KEY, pkce_verifier);

    // Redirect to the auth page
    leptos_axum::redirect(cx, auth_url.as_ref());
    Ok(())
}

#[server(GetToken, "/api")]
pub async fn get_token(cx: Scope, auth_code: String) -> Result<User, ServerFnError> {
    // Once the user has been redirected to the redirect URL, you'll have access to the
    // authorization code. For security reasons, your code should verify that the `state`
    // parameter returned by the server matches `csrf_state`.

    let auth_session = get_session(cx)?;

    let nonce = oidc::Nonce::new(auth_session.session.get(NONCE_KEY).unwrap_or("".to_string()));
    let pkce_verifier = oidc::PkceCodeVerifier::new(auth_session.session.get(PKCE_KEY).unwrap_or("".to_string()));

    let client = get_auth_client().await?;

    // Now you can exchange it for an access token and ID token.
    let token_response =
        client
            .exchange_code(oidc::AuthorizationCode::new(auth_code))
            // Set the PKCE code verifier.
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client).await?;

    // Extract the ID token claims after verifying its authenticity and nonce.
    let id_token = token_response.id_token().ok_or(ServerFnError::ServerError("Error getting id token.".to_owned()))?;
    println!("id_token: {:?}", id_token);
    let claims = id_token.claims(&client.id_token_verifier(), &nonce)?;
    println!("claims: {:?}", claims);

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
    let userinfo: oidc::core::CoreUserInfoClaims = client
        .user_info(token_response.access_token().to_owned(), None)
        .map_err(|err| ServerFnError::ServerError("No user info endpoint: ".to_owned() + &err.to_string()))?
        .request_async(async_http_client).await
        .map_err(|err| ServerFnError::ServerError("Failed requesting user info: ".to_owned() + &err.to_string()))?;

    auth_session.session.set(ID_TOKEN_KEY, token_response.clone());

    println!("stored user = {}", claims.subject().to_string());

    // See the OAuth2TokenResponse trait for a listing of other available fields such as
    // access_token() and refresh_token().

    // Redirect to home page
    leptos_axum::redirect(cx, "/");

    Ok(User {
        id: claims.subject().to_string(),
        anonymous: false,
        username: userinfo.preferred_username().unwrap().to_string(),
    })
}

#[server(GetUser, "/api")]
pub async fn get_user(cx: Scope) -> Result<User, ServerFnError> {
    let session = get_session(cx)?;
    let token_response: oidc::core::CoreTokenResponse = session.session.get(ID_TOKEN_KEY).ok_or(ServerFnError::ServerError(String::from("Not authenticated.")))?;
    let nonce = oidc::Nonce::new(session.session.get(NONCE_KEY).unwrap_or("".to_string()));

    let client = get_auth_client().await?;

    // Extract the ID token claims, authenticity and nonce already verified in auth callback
    let id_token = token_response.id_token().ok_or(ServerFnError::ServerError("Error getting id token.".to_owned()))?;
    println!("id_token: {:?}", id_token);
    let claims = id_token.claims(&client.id_token_verifier(), &nonce)?;
    println!("claims: {:?}", claims);

    Ok(User {
        id: claims.subject().to_string(),
        anonymous: false,
        username: claims.preferred_username().unwrap().to_string(),
    })
}

#[server(EndSession, "/api")]
pub async fn end_session(cx: Scope) -> Result<(), ServerFnError> {
    println!("Logout.");

    let session = get_session(cx)?;
    let token_response: oidc::core::CoreTokenResponse = session.session.get(ID_TOKEN_KEY).ok_or(ServerFnError::ServerError(String::from("Not authenticated.")))?;

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
        .set_post_logout_redirect_uri(get_logout_redirect()?);

    leptos_axum::redirect(cx, logout_request.http_get_url().to_string().as_str());

    session.session.remove(ID_TOKEN_KEY);

    Ok(())
}

/// Auth callback component
#[component]
pub fn AuthCallback(
    cx: Scope) -> impl IntoView
{
    let state = expect_context::<GlobalState>(cx);

    let query = use_query_map(cx);
    let code = move || query().get("code").unwrap().to_owned();
    let token_resource = create_blocking_resource(cx, || (), move |_| get_token(cx, code()));
    view! { cx,
        <h1>"Auth Callback"</h1>
        <Suspense fallback=|| ()>
            {move || {
                token_resource
                    .with(
                        cx,
                        |token| {
                            let Ok(user) = token else {
                                return view! { cx, <div>"Nothing"</div> }.into_view(cx);
                            };

                            state.user.set(user.clone());
                            leptos_router::Redirect(cx, RedirectProps { path: "/", options: None});
                            view! { cx, <div>"Success!"</div> }.into_view(cx)
                        },
                    )
            }}
        </Suspense>
    }
}

