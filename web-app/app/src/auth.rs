use std::env;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use leptos_router::params::Params;
#[cfg(feature = "ssr")]
use openidconnect as oidc;
#[cfg(feature = "ssr")]
use openidconnect::reqwest::*;
#[cfg(feature = "ssr")]
use openidconnect::{OAuth2TokenResponse, TokenResponse};
use web_sys::MouseEvent;
use crate::app::GlobalState;
use crate::errors::AppError;
use crate::icons::LoadingIcon;
use crate::unpack::SuspenseUnpack;
use crate::user::User;
#[cfg(feature = "ssr")]
use crate::{
    app::ssr::{get_db_pool, get_session},
    constants::SITE_ROOT,
    user::ssr::{create_or_update_user, SqlUser}
};
use crate::navigation_bar::get_current_path;

pub const BASE_URL_ENV: &str = "LEPTOS_SITE_ADDR";
pub const OIDC_ISSUER_URL_ENV: &str = "OIDC_ISSUER_ADDR";
pub const AUTH_CLIENT_ID_ENV: &str = "AUTH_CLIENT_ID";
pub const AUTH_CLIENT_SECRET_ENV: &str = "AUTH_CLIENT_SECRET";
pub const AUTH_CALLBACK_ROUTE: &str = "/authback";
pub const PKCE_KEY: &str = "pkce";
pub const NONCE_KEY: &str = "nonce";
pub const OIDC_TOKEN_KEY: &str = "oidc_token";
pub const OIDC_USERNAME_KEY: &str = "oidc_username";
pub const REDIRECT_URL_KEY: &str = "redirect";

#[derive(Params, Debug, PartialEq, Clone)]
pub struct OAuthParams {
    pub code: Option<String>,
    pub state: Option<String>,
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use super::*;
    use crate::app::ssr::get_user_lock_cache;
    use crate::errors::AppError;
    use crate::user::User;
    use axum_session_sqlx::SessionPgPool;
    use openidconnect::core::CoreTokenResponse;
    use openidconnect::{AdditionalProviderMetadata, NonceVerifier, RequestTokenError};
    use reqwest::Client;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use sqlx::PgPool;

    pub type AuthSession = axum_session_auth::AuthSession<User, i64, SessionPgPool, PgPool>;

    /// A no-op NonceVerifier implementation.
    struct NoNonceVerifier;

    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    pub struct AccountProviderMetadata
    {
        pub account_metadata: String,
    }

    impl NonceVerifier for NoNonceVerifier {
        fn verify(self, _nonce: Option<&openidconnect::Nonce>) -> std::result::Result<(), String> {
            Ok(())
        }
    }

    impl AdditionalProviderMetadata for AccountProviderMetadata {}

    pub fn get_issuer_url() -> Result<String, AppError> {
        Ok(env::var(OIDC_ISSUER_URL_ENV)?)
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
        let issuer_url = oidc::IssuerUrl::new(get_issuer_url()?)?;

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

    pub async fn check_user() -> Result<User, AppError> {
        let user = check_refresh_token().await?;
        user.ok_or(AppError::NotAuthenticated)
    }

    pub fn reload_user(user_id: i64) -> Result<(), AppError> {
        let auth_session = get_session()?;
        auth_session.cache_clear_user(user_id);
        Ok(())
    }
    
    fn get_nonce(auth_session: &AuthSession) -> Option<oidc::Nonce> {
        match auth_session.session.get::<String>(NONCE_KEY) {
            Some(nonce) if !nonce.is_empty() => Some(oidc::Nonce::new(nonce)),
            _ => None,
        }
    }

    pub async fn check_refresh_token() -> Result<Option<User>, AppError> {
        let auth_session = get_session()?;
        if let Some(user) = &auth_session.current_user {
            let user_lock = get_user_lock_cache()?.get_user_lock(user.user_id).await;

            // Lock the mutex for this user
            let _lock = user_lock.lock().await;

            let token_response: CoreTokenResponse = auth_session
                .session
                .get(OIDC_TOKEN_KEY)
                .ok_or(AppError::new("Token missing, cannot check validity."))?;

            let id_token = token_response.id_token().ok_or(AppError::new("Id token missing."))?;

            let client = get_auth_client().await?;

            let claims = match get_nonce(&auth_session) {
                Some(nonce) => id_token.claims(&client.id_token_verifier(), &nonce),
                None => id_token.claims(&client.id_token_verifier(), NoNonceVerifier),
            };
            match claims {
                Err(openidconnect::ClaimsVerificationError::Expired(_)) => {
                    log::debug!("Id token expired, refresh tokens.");
                    auth_session.session.remove(NONCE_KEY);
                    let refresh_token = token_response.refresh_token().ok_or(AppError::new("Error getting refresh token."))?;
                    let token_response = client
                        .exchange_refresh_token(refresh_token)
                        .request_async(async_http_client)
                        .await;

                    match token_response {
                        Ok(token_response) => {
                            let sql_user = process_token_response(token_response, auth_session.clone(), client).await?;
                            let db_pool = get_db_pool()?;
                            let user = User::get(sql_user.user_id, &db_pool).await;
                            log::debug!("Logged in as {:?}", sql_user);
                            auth_session.cache_clear_user(sql_user.user_id);
                            Ok(user)
                        }
                        Err(e) => {
                            match e {
                                RequestTokenError::ServerResponse(response) => {
                                    log::error!("Failed to refresh token: server returned an error: {:?}", response);
                                }
                                RequestTokenError::Request(http_err) => {
                                    log::error!("Failed to refresh token: HTTP request failed: {:?}", http_err);
                                }
                                RequestTokenError::Parse(err, body) => {
                                    log::error!("Failed to refresh token: failed to parse response: {:?}. Response body: {:?}", err, body);
                                }
                                RequestTokenError::Other(msg) => {
                                    log::error!("Failed to refresh token: other error: {:?}", msg);
                                }
                            }
                            auth_session.logout_user();
                            Ok(None)
                        }
                    }
                },
                Err(e) => {
                    log::error!("Unexpected error while getting claims: {e}");
                    auth_session.session.remove(NONCE_KEY);
                    auth_session.session.remove(OIDC_TOKEN_KEY);
                    auth_session.logout_user();
                    Ok(None)
                },
                Ok(claims) => {
                    log::debug!("Id token valid until {}", claims.expiration());
                    Ok(auth_session.current_user)
                },
            }
        } else {
            log::debug!("Not logged in.");
            Ok(None)
        }
    }

    /// process the input token response, upsert the corresponding user and returns it
    pub async fn process_token_response(
        token_response: CoreTokenResponse,
        auth_session: AuthSession,
        client: oidc::core::CoreClient,
    ) -> Result<SqlUser, AppError> {
        // Extract the ID token claims after verifying its authenticity and nonce.
        let id_token = token_response
            .id_token()
            .ok_or(AppError::new("Id token missing."))?;

        let claims = match get_nonce(&auth_session) {
            Some(nonce) => id_token.claims(&client.id_token_verifier(), &nonce),
            None => id_token.claims(&client.id_token_verifier(), NoNonceVerifier),
        };

        let claims = match claims {
            Ok(claims) => claims,
            Err(e) => {
                log::error!("Failed to get claims: {e}.");
                return Err(e.into());
            }
        };

        // Verify the access token hash to ensure that the access token hasn't been substituted for another user's.
        if let Some(expected_access_token_hash) = claims.access_token_hash() {
            let actual_access_token_hash = oidc::AccessTokenHash::from_token(
                token_response.access_token(),
                &id_token.signing_alg()?,
            )?;
            if actual_access_token_hash != *expected_access_token_hash {
                return Err(AppError::new("Invalid access token"));
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

        auth_session.session.remove(OIDC_TOKEN_KEY);
        auth_session.session.set(OIDC_TOKEN_KEY, token_response.clone());

        let oidc_id = claims.subject().to_string();
        let db_pool = get_db_pool()?;

        let username: String = claims.preferred_username().ok_or(AppError::new("Username missing from token"))?.to_string();
        let email: String = claims.email().ok_or(AppError::new("Email missing from token"))?.to_string();
        let user = create_or_update_user(&oidc_id, &username, &email, &db_pool).await?;

        Ok(user)
    }

    pub async fn redirect_to_auth_provider(redirect_url: String) -> Result<(), AppError> {
        let client = get_auth_client().await?;

        // Generate the full authorization URL.
        let (auth_url, _csrf_token, nonce) = client
            .authorize_url(
                oidc::core::CoreAuthenticationFlow::AuthorizationCode,
                oidc::CsrfToken::new_random,
                oidc::Nonce::new_random,
            )
            // TODO remove scopes?
            // Set the desired scopes.
            //.add_scope(oidc::Scope::new("read".to_string()))
            //.add_scope(oidc::Scope::new("write".to_string()))
            .url();

        let auth_session = get_session()?;

        auth_session.session.set(NONCE_KEY, nonce);
        auth_session.session.set(REDIRECT_URL_KEY, redirect_url);

        // Redirect to the auth page
        leptos_axum::redirect(auth_url.as_ref());
        Ok(())
    }

    pub async fn navigate_to_user_account() -> Result<(), AppError> {
        let issuer_url = get_issuer_url()?;
        let client = Client::new();
        // Fetch the discovery endpoint data
        let response = client.get(&issuer_url).send().await.map_err(AppError::new)?.json::<Value>().await.map_err(AppError::new)?;

        // Obtain the account service url from it
        let account_service_url = response.get("account-service").and_then(|v| v.as_str()).ok_or(AppError::new("Account-service missing from provider"))?;

        // Redirect to the user account page
        leptos_axum::redirect(account_service_url);
        Ok(())
    }
}

#[server]
pub async fn login(redirect_url: String) -> Result<Option<User>, ServerFnError<AppError>> {
    let current_user = get_user().await;

    if let Ok(Some(current_user)) = current_user
    {
        return Ok(Some(current_user));
    }

    ssr::redirect_to_auth_provider(redirect_url).await?;

    Ok(None)
}

#[server]
pub async fn navigate_to_user_account() -> Result<(), ServerFnError<AppError>> {
    ssr::navigate_to_user_account().await?;
    Ok(())
}

#[server]
pub async fn authenticate_user(auth_code: String) -> Result<(), ServerFnError<AppError>> {
    // Once the user has been redirected to the redirect URL, you'll have access to the
    // authorization code. For security reasons, your code should verify that the `state`
    // parameter returned by the server matches `csrf_state`.

    let auth_session = get_session()?;

    let redirect_url = auth_session
        .session
        .get(REDIRECT_URL_KEY)
        .unwrap_or(String::from(SITE_ROOT));

    let client = ssr::get_auth_client().await?;

    // Now you can exchange it for an access token and ID token.
    let token_response = client
        .exchange_code(oidc::AuthorizationCode::new(auth_code))
        .request_async(async_http_client)
        .await
        .map_err(AppError::from)?;

    let sql_user = ssr::process_token_response(token_response, auth_session.clone(), client).await?;
    auth_session.login_user(sql_user.user_id);
    auth_session.remember_user(true);

    leptos_axum::redirect(redirect_url.as_ref());
    Ok(())
}

#[server]
pub async fn get_user() -> Result<Option<User>, ServerFnError<AppError>> {
    let user = ssr::check_refresh_token().await?;
    Ok(user)
}

#[server]
pub async fn end_session(redirect_url: String) -> Result<(), ServerFnError<AppError>> {
    log::debug!("Logout, redirect_url: {redirect_url}");

    let auth_session = get_session()?;
    let token_response: oidc::core::CoreTokenResponse =
        auth_session
            .session
            .get(OIDC_TOKEN_KEY)
            .ok_or(AppError::InternalServerError(String::from("Not authenticated.")))?;

    let id_token = token_response.id_token().ok_or(AppError::AuthenticationError(String::from("Id token missing.")))?;

    let logout_provider_metadata =
        oidc::ProviderMetadataWithLogout::discover_async(
            oidc::IssuerUrl::new(
                ssr::get_issuer_url()?
            ).map_err(|e| AppError::new(e))?, async_http_client
        )
            .await
            .map_err(AppError::from)?;

    let logout_endpoint: &Option<oidc::EndSessionUrl> = &logout_provider_metadata
        .additional_metadata()
        .end_session_endpoint;

    let logout_endpoint_url = match logout_endpoint {
        Some(url) => Ok(url.clone()),
        None => Err(AppError::AuthenticationError(String::from("Cannot get logout endpoint."))),
    }?;

    let logout_request = oidc::LogoutRequest::from(logout_endpoint_url)
        .set_client_id(ssr::get_client_id()?)
        .set_id_token_hint(id_token)
        .set_post_logout_redirect_uri(oidc::PostLogoutRedirectUrl::new(redirect_url).map_err(AppError::from)?);

    leptos_axum::redirect(logout_request.http_get_url().to_string().as_str());

    auth_session.session.remove(OIDC_TOKEN_KEY);
    auth_session.logout_user();

    Ok(())
}

/// Guard for a component requiring a login. If the user is logged in, the children of this component will be rendered
/// Otherwise, it will be replaced by a form/button with the same appearance redirecting to a login screen.
#[component]
pub fn LoginGuardButton<
    F: Fn(&User) -> IV + Clone + Send + Sync + 'static,
    IV: IntoView + 'static,
    G: Fn(RwSignal<String>) + Send + Sync + 'static
>(
    #[prop(default = "")]
    login_button_class: &'static str,
    #[prop(into)]
    login_button_content: ViewFn,
    redirect_path_fn: &'static G,
    children: F,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let children = StoredValue::new(children);
    let login_button_content = StoredValue::new(login_button_content);

    view! {
        <Transition fallback=move || view! { <LoadingIcon/> }>
        {
            move || Suspend::new(async move {
                match &state.user.await {
                    Ok(Some(user)) => children.with_value(|children| children(user)).into_any(),
                    _ => {
                        let login_button_view = login_button_content.with_value(|content| content.run());
                        view! { <LoginButton class=login_button_class redirect_path_fn>{login_button_view}</LoginButton> }.into_any()
                    },
                }
            })
        }
        </Transition>
    }.into_any()
}

/// Login guarded button component. If the user is logged in, a button with the given class and action will be rendered.
/// Otherwise, the button will redirect the user to a login screen.
#[component]
pub fn LoginGuardedButton<A, IV>(
    #[prop(into)]
    button_class: Signal<&'static str>,
    button_action: A,
    children: TypedChildrenFn<IV>,
) -> impl IntoView
where
    A: Fn(MouseEvent) -> () + Clone + Send + Sync + 'static,
    IV: IntoView + 'static
{
    let state = expect_context::<GlobalState>();
    let children = StoredValue::new(children.into_inner());
    let button_action = StoredValue::new(button_action);
    view! {
        <Transition fallback=move || view! { <LoadingIcon/> }>
        {
            move || Suspend::new(async move {
                let children = children.get_value();
                match &state.user.await {
                    Ok(Some(_)) => view! {
                        <button
                            class=button_class
                            aria-haspopup="dialog"
                            on:click=button_action.get_value()
                        >
                            {children()}
                        </button>
                    }.into_any(),
                    _ => view! { <LoginButton class=button_class redirect_path_fn=&get_current_path>{children()}</LoginButton> }.into_any(),
                }
            })
        }
        </Transition>
    }
}

#[component]
fn LoginButton<
    F: Fn(RwSignal<String>) + Send + Sync + 'static
>(
    #[prop(into)]
    class: Signal<&'static str>,
    redirect_path_fn: &'static F,
    children: Children,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let redirect_path = RwSignal::new(String::default());

    view! {
        <ActionForm action=state.login_action attr:class="flex items-center">
            <input type="text" name="redirect_url" class="hidden" value=redirect_path/>
            <button type="submit" class=class on:click=move |_| redirect_path_fn(redirect_path)>
                {children()}
            </button>
        </ActionForm>
    }.into_any()
}

/// Auth callback component
#[component]
pub fn AuthCallback() -> impl IntoView {
    let query = use_query_map();
    let code = move || query.read_untracked().get("code").unwrap().to_string();
    let auth_resource = Resource::new_blocking(
        || (),
        move |_| {
            log::trace!("Authenticate user.");
            authenticate_user(code())
        }
    );

    view! {
        <SuspenseUnpack
            resource=auth_resource
            let:_auth_result
        >
            {
                log::debug!("Authenticated successfully");
            }
        </SuspenseUnpack>
    }.into_any()
}
