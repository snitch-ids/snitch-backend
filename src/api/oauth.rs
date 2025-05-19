use actix_web::web::Redirect;
use actix_web::{get, post, web, web::Form, web::Json, Responder};
use log::{error, info, warn};
use openidconnect::core::{
    CoreAuthDisplay, CoreClaimName, CoreClaimType, CoreClient, CoreClientAuthMethod, CoreGrantType,
    CoreIdTokenClaims, CoreIdTokenVerifier, CoreJsonWebKey, CoreJweContentEncryptionAlgorithm,
    CoreJweKeyManagementAlgorithm, CoreResponseMode, CoreResponseType, CoreRevocableToken,
    CoreSubjectIdentifierType,
};
use openidconnect::reqwest;
use openidconnect::{
    AdditionalProviderMetadata, AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret,
    CsrfToken, IssuerUrl, Nonce, OAuth2TokenResponse, ProviderMetadata, RedirectUrl, RevocationUrl,
    Scope,
};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct RevocationEndpointProviderMetadata {
    revocation_endpoint: String,
}
impl AdditionalProviderMetadata for RevocationEndpointProviderMetadata {}

type GoogleProviderMetadata = ProviderMetadata<
    RevocationEndpointProviderMetadata,
    CoreAuthDisplay,
    CoreClientAuthMethod,
    CoreClaimName,
    CoreClaimType,
    CoreGrantType,
    CoreJweContentEncryptionAlgorithm,
    CoreJweKeyManagementAlgorithm,
    CoreJsonWebKey,
    CoreResponseMode,
    CoreResponseType,
    CoreSubjectIdentifierType,
>;

#[derive(Debug, Deserialize, Serialize)]
struct OAuth2TokenResponseForm {
    code: AuthorizationCode,
    state: CsrfToken,
    scope: String,
    authuser: String,
    prompt: String,
}

#[get("/oauth_done")]
pub async fn oauth_done(
    oauth2token_response_form: web::Query<OAuth2TokenResponseForm>,
) -> Result<impl Responder, actix_web::Error> {
    info!("{:?}", oauth2token_response_form);
    println!(
        "Google returned the following code:\n{}\n",
        oauth2token_response_form.code.secret()
    );
    println!(
        "Google returned the following state:\n{} (expected )\n",
        oauth2token_response_form.state.secret(),
    );

    let google_client_id = ClientId::new(
        env::var("SNITCH_GOOGLE_CLIENT_ID")
            .expect("Missing the GOOGLE_CLIENT_ID environment variable."),
    );
    let google_client_secret = ClientSecret::new(
        env::var("SNITCH_GOOGLE_CLIENT_SECRET")
            .expect("Missing the GOOGLE_CLIENT_SECRET environment variable."),
    );
    let issuer_url = IssuerUrl::new("https://accounts.google.com".to_string())
        .map_err(|err| actix_web::error::ErrorInternalServerError(err))?;

    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|err| actix_web::error::ErrorInternalServerError(err))
        .unwrap();

    let provider_metadata = GoogleProviderMetadata::discover_async(issuer_url, &http_client)
        .await
        .unwrap_or_else(|err| {
            error!("Failed to discover OpenID Provider");
            unreachable!();
        });

    let revocation_endpoint = provider_metadata
        .additional_metadata()
        .revocation_endpoint
        .clone();
    println!(
        "Discovered Google revocation endpoint: {}",
        revocation_endpoint
    );

    // Set up the config for the Google OAuth2 process.
    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        google_client_id,
        Some(google_client_secret),
    )
    // This example will be running its own server at localhost:8080.
    // See below for the server implementation.
    .set_redirect_uri(
        RedirectUrl::new("http://localhost:8081/oauth_done".to_string()).unwrap_or_else(|err| {
            error!("Invalid redirect URL");
            unreachable!();
        }),
    )
    // Google supports OAuth 2.0 Token Revocation (RFC-7009)
    .set_revocation_url(
        RevocationUrl::new(revocation_endpoint).unwrap_or_else(|err| {
            error!("Invalid revocation endpoint URL");
            unreachable!();
        }),
    );

    let code = oauth2token_response_form.code.clone();
    // Exchange the code with a token.
    let token_response = client
        .exchange_code(code)
        .unwrap_or_else(|err| {
            error!("No user info endpoint");
            unreachable!();
        })
        .request_async(&http_client)
        .await
        .unwrap_or_else(|err| {
            error!("Failed to contact token endpoint");
            unreachable!();
        });
    warn!("{:?}", token_response);
    Ok("token_response")
}

#[get("/oauth")]
pub async fn oauth() -> Result<impl Responder, actix_web::Error> {
    let google_client_id = ClientId::new(
        env::var("SNITCH_GOOGLE_CLIENT_ID")
            .expect("Missing the GOOGLE_CLIENT_ID environment variable."),
    );
    let google_client_secret = ClientSecret::new(
        env::var("SNITCH_GOOGLE_CLIENT_SECRET")
            .expect("Missing the GOOGLE_CLIENT_SECRET environment variable."),
    );
    let issuer_url = IssuerUrl::new("https://accounts.google.com".to_string())
        .map_err(|err| actix_web::error::ErrorInternalServerError(err))?;

    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|err| actix_web::error::ErrorInternalServerError(err))?;

    let provider_metadata = GoogleProviderMetadata::discover_async(issuer_url, &http_client)
        .await
        .unwrap_or_else(|err| {
            error!("Failed to discover OpenID Provider");
            unreachable!();
        });

    let revocation_endpoint = provider_metadata
        .additional_metadata()
        .revocation_endpoint
        .clone();
    println!(
        "Discovered Google revocation endpoint: {}",
        revocation_endpoint
    );

    // Set up the config for the Google OAuth2 process.
    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        google_client_id,
        Some(google_client_secret),
    )
    // This example will be running its own server at localhost:8080.
    // See below for the server implementation.
    .set_redirect_uri(
        RedirectUrl::new("http://localhost:8081/oauth_done".to_string()).unwrap_or_else(|err| {
            error!("Invalid redirect URL");
            unreachable!();
        }),
    )
    // Google supports OAuth 2.0 Token Revocation (RFC-7009)
    .set_revocation_url(
        RevocationUrl::new(revocation_endpoint).unwrap_or_else(|err| {
            error!("Invalid revocation endpoint URL");
            unreachable!();
        }),
    );

    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, csrf_state, nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        // This example is requesting access to the "calendar" features and the user's profile.
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    Ok(Redirect::to(authorize_url.to_string()))
}
