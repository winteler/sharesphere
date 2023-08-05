use cfg_if::cfg_if;
use leptos::*;

cfg_if! {
if #[cfg(feature = "ssr")] {
    use openidconnect as oidc;
    use anyhow::anyhow;

    use openidconnect::reqwest::http_client;
    use openidconnect::url::Url;

    // Use OpenID Connect Discovery to fetch the provider metadata.
    use openidconnect::{OAuth2TokenResponse, TokenResponse};
}}

#[cfg(feature = "ssr")]
pub async fn get_auth_client() -> Result<(), anyhow::Error> {

    let provider_metadata = oidc::core::CoreProviderMetadata::discover(
        &oidc::IssuerUrl::new("https://accounts.example.com".to_string())?,
        http_client,
    )?;

    // Create an OpenID Connect client by specifying the client ID, client secret, authorization URL
    // and token URL.
    let client =
        oidc::core::CoreClient::from_provider_metadata(
            provider_metadata,
            oidc::ClientId::new("client_id".to_string()),
            Some(oidc::ClientSecret::new("client_secret".to_string())),
        )
    // Set the URL the user will be redirected to after the authorization process.
            .set_redirect_uri(oidc::RedirectUrl::new("http://redirect".to_string())?);

    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = oidc::PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (auth_url, csrf_token, nonce) = client
        .authorize_url(
            oidc::core::CoreAuthenticationFlow::AuthorizationCode,
            oidc::CsrfToken::new_random,
            oidc::Nonce::new_random,
        )
    // Set the desired scopes.
        .add_scope(oidc::Scope::new("read".to_string()))
        .add_scope(oidc::Scope::new("write".to_string()))
    // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();

    // This is the URL you should redirect the user to, in order to trigger the authorization
    // process.
    println!("Browse to: {}", auth_url);

    // Once the user has been redirected to the redirect URL, you'll have access to the
    // authorization code. For security reasons, your code should verify that the `state`
    // parameter returned by the server matches `csrf_state`.

    // Now you can exchange it for an access token and ID token.
    let token_response =
        client
            .exchange_code(oidc::AuthorizationCode::new("some authorization code".to_string()))
    // Set the PKCE code verifier.
            .set_pkce_verifier(pkce_verifier)
            .request(http_client)?;

    // Extract the ID token claims after verifying its authenticity and nonce.
    let id_token = token_response
        .id_token()
        .ok_or_else(|| anyhow!("Server did not return an ID token"))?;
    let claims = id_token.claims(&client.id_token_verifier(), &nonce)?;

    // Verify the access token hash to ensure that the access token hasn't been substituted for
    // another user's.
    if let Some(expected_access_token_hash) = claims.access_token_hash() {
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
    );

    // If available, we can use the UserInfo endpoint to request additional information.

    // The user_info request uses the AccessToken returned in the token response. To parse custom
    // claims, use UserInfoClaims directly (with the desired type parameters) rather than using the
    // CoreUserInfoClaims type alias.
    let userinfo: oidc::core::CoreUserInfoClaims = client
        .user_info(token_response.access_token().to_owned(), None)
        .map_err(|err| anyhow!("No user info endpoint: {:?}", err))?
        .request(http_client)
        .map_err(|err| anyhow!("Failed requesting user info: {:?}", err))?;

    // See the OAuth2TokenResponse trait for a listing of other available fields such as
    // access_token() and refresh_token().
    Ok(())
}

/// Navigation bar component
#[component]
pub fn AuthCallback(
    cx: Scope) -> impl IntoView
{
    view! { cx,
        <div class="alert alert-warning">
            <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6 stroke-white" fill="none" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            <span>"Warning: auth isn't implemented yet!"</span>
        </div>
    }
}

