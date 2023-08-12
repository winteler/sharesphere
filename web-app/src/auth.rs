use cfg_if::cfg_if;
use std::env;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

pub const BASE_URL_ENV : &str = "LEPTOS_SITE_ADDR";
pub const AUTH_ROUTE : &str = "/auth";
pub const AUTH_CALLBACK_ROUTE : &str = "/authback";

cfg_if! {
if #[cfg(feature = "ssr")] {

    use sqlx::{PgPool, ConnectOptions, postgres::{PgPoolOptions, PgConnectOptions}};
    use axum_session_auth::{SessionPgPool, Authentication, HasPermission};

    use anyhow::anyhow;

    use openidconnect as oidc;
    use openidconnect::reqwest::*;
    use openidconnect::url::Url;
    // Use OpenID Connect Discovery to fetch the provider metadata.
    use openidconnect::{OAuth2TokenResponse, TokenResponse};

    pub type AuthSession = axum_session_auth::AuthSession<User, i64, SessionPgPool, PgPool>;
}}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: -1,
        }
    }
}

cfg_if! {
if #[cfg(feature = "ssr")] {
    use async_trait::async_trait;

    #[async_trait]
    impl Authentication<User, i64, PgPool> for User {
        async fn load_user(userid: i64, pool: Option<&PgPool>) -> Result<User, anyhow::Error> {
            Ok(User::default())
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
}}

#[cfg(feature = "ssr")]
pub async fn get_auth_client() -> Result<oidc::core::CoreClient, ServerFnError> {

    let base_url = env::var(BASE_URL_ENV)?;
    let redirect_url = String::from("http://") + &base_url + AUTH_CALLBACK_ROUTE;
    let issuer_url = oidc::IssuerUrl::new("http://127.0.0.1:8080/realms/project".to_string()).expect("Invalid issuer URL");

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

#[server(GetAuthUrl, "/api")]
pub async fn get_auth_url(cx: Scope) -> Result<String, ServerFnError> {

    println!("get client");
    let client = get_auth_client().await?;

    println!("generate url");
    // Generate the full authorization URL.
    let (auth_url, csrf_token, nonce) = client
        .authorize_url(
            oidc::core::CoreAuthenticationFlow::AuthorizationCode,
            oidc::CsrfToken::new_random,
            oidc::Nonce::new_random,
        )
        // Set the desired scopes.
        //.add_scope(oidc::Scope::new("read".to_string()))
        //.add_scope(oidc::Scope::new("write".to_string()))
        .url();

    // This is the URL you should redirect the user to, in order to trigger the authorization
    // process.
    println!("Browse to: {}", auth_url);

    Ok(auth_url.to_string())
}

#[server(GetToken, "/api")]
pub async fn get_token(cx: Scope, auth_code: String) -> Result<bool, ServerFnError> {
    // Once the user has been redirected to the redirect URL, you'll have access to the
    // authorization code. For security reasons, your code should verify that the `state`
    // parameter returned by the server matches `csrf_state`.

    println!("Get token, auth_code = {auth_code}");

    let client = get_auth_client().await?;

    println!("Got client");

    // Now you can exchange it for an access token and ID token.
    let token_response =
        client
            .exchange_code(oidc::AuthorizationCode::new(auth_code))
    // Set the PKCE code verifier.
            //.set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client).await?;

    println!("Got token_response");

    // Extract the ID token claims after verifying its authenticity and nonce.
    let id_token = token_response.id_token().ok_or(ServerFnError::Args("Error getting id token.".to_owned()))?;
    println!("id_token: {:?}", id_token);
    //let claims = id_token.claims(&client.id_token_verifier(), &nonce)?;

    // Verify the access token hash to ensure that the access token hasn't been substituted for
    // another user's.
    /*if let Some(expected_access_token_hash) = claims.access_token_hash() {
        let actual_access_token_hash = oidc::AccessTokenHash::from_token(
            token_response.access_token(),
            &id_token.signing_alg()?
        )?;
        if actual_access_token_hash != *expected_access_token_hash {
            return Err(anyhow!("Invalid access token"));
        }
    }

    // The authenticated user's identity is now available. See the IdTokenClaims struct for a
    // complete listing of the available claims.
    println!(
        "User {} with e-mail address {} has authenticated successfully",
        claims.subject().as_str(),
        claims.email().map(|email| email.as_str()).unwrap_or("<not provided>"),
    );*/

    // If available, we can use the UserInfo endpoint to request additional information.

    // The user_info request uses the AccessToken returned in the token response. To parse custom
    // claims, use UserInfoClaims directly (with the desired type parameters) rather than using the
    // CoreUserInfoClaims type alias.
    let userinfo: oidc::core::CoreUserInfoClaims = client
        .user_info(token_response.access_token().to_owned(), None)
        .map_err(|err| ServerFnError::Args("No user info endpoint: ".to_owned() + &err.to_string()))?
        .request_async(async_http_client).await
        .map_err(|err| ServerFnError::Args("Failed requesting user info: ".to_owned() + &err.to_string()))?;

    // See the OAuth2TokenResponse trait for a listing of other available fields such as
    // access_token() and refresh_token().
    Ok(true)
}

/// Navigation bar component
#[component]
pub fn Auth(
    cx: Scope) -> impl IntoView
{
    let auth_url_resource = create_resource(cx, || (), move |_| get_auth_url(cx));

    view! { cx,
        <h1>"You got to the redirect, yay! "</h1>
        <Suspense fallback=move || {
            view! { cx, <div>"Loading..."</div> }
        }>
            {move || match auth_url_resource.read(cx) {
                None => view! { cx, <div>"Loading..."</div> }.into_view(cx),
                Some(Err(e)) => view! { cx, <div>{e.to_string()}</div> }.into_view(cx),
                Some(Ok(auth_url)) => view! { cx, <a href=auth_url>"Start Login"</a> }.into_view(cx),
            }}
        </Suspense>
    }
}

/// Auth callback component
#[component]
pub fn AuthCallback(
    cx: Scope) -> impl IntoView
{
    let query = move || use_query_map(cx).get();
    let code = query().get("code").unwrap().to_owned();
    let token_resource = create_blocking_resource(cx, || (), move |_| get_token(cx, code.clone()));
    view! { cx,
        <h1>"Auth Callback"</h1>
        <Suspense fallback=|| ()>
            {move || {
                token_resource
                    .with(
                        cx,
                        |token| {
                            let Ok(auth_complete) = token else {
                            return view! { cx, <div>"Nothing"</div> }.into_view(cx);
                        };
                            view! { cx,
                                <div>"Token Received: " {format!("{:?}", auth_complete)}</div>
                            }
                                .into_view(cx)
                        },
                    )
            }}
        </Suspense>
    }
}

