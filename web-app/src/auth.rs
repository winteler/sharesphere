use cfg_if::cfg_if;
use std::env;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

use crate::app::{GlobalState};

pub const BASE_URL_ENV : &str = "LEPTOS_SITE_ADDR";
pub const AUTH_CALLBACK_ROUTE : &str = "/authback";
pub const PKCE_KEY : &str = "pkce";
pub const NONCE_KEY : &str = "nonce";

cfg_if! {
if #[cfg(feature = "ssr")] {

    use sqlx::{PgPool};
    use axum_session_auth::{SessionPgPool, Authentication};

    use openidconnect as oidc;
    use openidconnect::reqwest::*;
    // Use OpenID Connect Discovery to fetch the provider metadata.
    use openidconnect::{OAuth2TokenResponse, TokenResponse};

    pub type AuthSession = axum_session_auth::AuthSession<User, i64, SessionPgPool, PgPool>;
}}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub anonymous: bool,
    pub username: String,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: -1,
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
    impl Authentication<User, i64, PgPool> for User {
        async fn load_user(_userid: i64, _pool: Option<&PgPool>) -> Result<User, anyhow::Error> {
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
pub async fn get_auth_client() -> Result<oidc::core::CoreClient, ServerFnError> {

    let base_url = env::var(BASE_URL_ENV)?;
    let redirect_url = String::from("http://") + &base_url + AUTH_CALLBACK_ROUTE;
    let issuer_url = oidc::IssuerUrl::new("http://127.0.0.1:8080/realms/project".to_string()).expect("Invalid issuer URL");

    println!("redirect url: {}", redirect_url);

    let provider_metadata = oidc::core::CoreProviderMetadata::discover_async(
        issuer_url,
        async_http_client
    ).await?;

    // Create an OpenID Connect client by specifying the client ID, client secret, authorization URL
    // and token URL.
    let client =
        oidc::core::CoreClient::from_provider_metadata(
            provider_metadata,
            oidc::ClientId::new("project-client".to_string()),
            Some(oidc::ClientSecret::new("psXCKGKe4E5pVHwneYRPizBv84CHKL32".to_string())),
        )
        // Set the URL the user will be redirected to after the authorization process.
        .set_redirect_uri(oidc::RedirectUrl::new(redirect_url)?);

    Ok(client)
}

#[server(StartAuth, "/api")]
pub async fn start_auth(cx: Scope) -> Result<(), ServerFnError> {

    println!("get client");
    let client = get_auth_client().await?;

    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = oidc::PkceCodeChallenge::new_random_sha256();

    println!("generate url");
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

    get_session(cx)?.session.set(NONCE_KEY, nonce);
    get_session(cx)?.session.set(PKCE_KEY, pkce_verifier);

    // This is the URL you should redirect the user to, in order to trigger the authorization
    // process.
    println!("Browse to: {}", auth_url);

    // and redirect to the home page
    leptos_axum::redirect(cx, auth_url.as_ref());
    Ok(())
}

#[server(GetToken, "/api")]
pub async fn get_token(cx: Scope, auth_code: String) -> Result<User, ServerFnError> {
    // Once the user has been redirected to the redirect URL, you'll have access to the
    // authorization code. For security reasons, your code should verify that the `state`
    // parameter returned by the server matches `csrf_state`.

    println!("Get token, auth_code = {auth_code}");

    let nonce = oidc::Nonce::new(get_session(cx)?.session.get(NONCE_KEY).unwrap_or("".to_string()));
    let pkce_verifier = oidc::PkceCodeVerifier::new(get_session(cx)?.session.get(PKCE_KEY).unwrap_or("".to_string()));

    let client = get_auth_client().await?;

    println!("Got client");

    // Now you can exchange it for an access token and ID token.
    let token_response =
        client
            .exchange_code(oidc::AuthorizationCode::new(auth_code))
            // Set the PKCE code verifier.
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client).await?;

    println!("Got token_response");

    // Extract the ID token claims after verifying its authenticity and nonce.
    let id_token = token_response.id_token().ok_or(ServerFnError::Args("Error getting id token.".to_owned()))?;
    println!("id_token: {:?}", id_token);
    let claims = id_token.claims(&client.id_token_verifier(), &nonce)?;

    // Verify the access token hash to ensure that the access token hasn't been substituted for
    // another user's.
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
        .map_err(|err| ServerFnError::Args("No user info endpoint: ".to_owned() + &err.to_string()))?
        .request_async(async_http_client).await
        .map_err(|err| ServerFnError::Args("Failed requesting user info: ".to_owned() + &err.to_string()))?;

    println!("userinfo = {:?}", userinfo);

    let user = User {
        id: -1,
        anonymous: false,
        username: userinfo.preferred_username().unwrap().to_string(),
    };

    get_session(cx)?.current_user = Some(user.clone());

    println!("stored user = {:?}", get_session(cx)?.current_user);

    // Redirect to home page
    //leptos_axum::redirect(cx, "/");

    // See the OAuth2TokenResponse trait for a listing of other available fields such as
    // access_token() and refresh_token().
    Ok(user)
}

#[server(Logout, "/api")]
pub async fn logout(cx: Scope) -> Result<(), ServerFnError> {
    // TODO: Perform logout in keycloak

    let session = get_session(cx)?;
    session.logout_user();

    Ok(())
}

#[server(GetUser, "/api")]
pub async fn get_user(cx: Scope) -> Result<Option<User>, ServerFnError> {
    let session = get_session(cx)?;
    Ok(session.current_user)
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
                            view! { cx, <div>"user: " {format!("{:?}", user)}</div>}.into_view(cx)
                        },
                    )
            }}
        </Suspense>
    }
}

