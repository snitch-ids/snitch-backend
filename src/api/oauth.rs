use crate::api::AppState;
use crate::errors::APIError;
use crate::model::user::User;
use actix_identity::Identity;
use actix_web::web::{redirect, Redirect};
use actix_web::{get, web, web::Json, HttpResponse, Responder};
use argonautica::utils::generate_random_base64_encoded_string;
use log::{error, info};
use openidconnect::core::{
    CoreAuthDisplay, CoreClaimName, CoreClaimType, CoreClient, CoreClientAuthMethod, CoreGrantType,
    CoreIdTokenClaims, CoreIdTokenVerifier, CoreJsonWebKey, CoreJweContentEncryptionAlgorithm,
    CoreJweKeyManagementAlgorithm, CoreResponseMode, CoreResponseType, CoreSubjectIdentifierType,
};
use openidconnect::reqwest;
use openidconnect::{
    AdditionalProviderMetadata, AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret,
    CsrfToken, IssuerUrl, Nonce, ProviderMetadata, RedirectUrl, RevocationUrl, Scope,
};
use reqwest::Request;
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
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
    oauth2token_response_form: web::Query<OAuth2TokenResponseForm>,
) -> Result<impl Responder, actix_web::Error> {
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

    let nonce = state
        .csrf_token
        .lock()
        .await
        .remove(&serde_json::to_string(&oauth2token_response_form.state)?)
        .unwrap();
    let nonce = serde_json::from_str::<Nonce>(&nonce)?;

    let id_token_verifier: CoreIdTokenVerifier = client.id_token_verifier();
    let id_token_claims: &CoreIdTokenClaims = token_response
        .extra_fields()
        .id_token()
        .expect("Server did not return an ID token")
        .claims(&id_token_verifier, &nonce)
        .unwrap_or_else(|err| {
            error!("Failed to verify ID token");
            unreachable!();
        });

    let email = id_token_claims
        .email()
        .map(|email| email.as_str())
        .unwrap_or("<not provided>");
    let user = User::new(
        email.to_string(),
        generate_random_base64_encoded_string(16).unwrap(),
    );

    let user = state
        .persist
        .lock()
        .await
        .add_user(user)
        .await
        .map_err(|err| {
            error!("Failed to add user: {}", err);
            APIError::InternalServerError
        })?;
    let x = HttpResponse::Found()
        // .cookie(cookie)
        .append_header(("Location", "http://localhost:8080/"))
        .finish();
    info!("login user: {:?}", user);

    // WIP: login causes crash:
    // Identity::login(&x.extensions(), user.user_id.to_string())?;
    Ok(x)
}

#[get("/oauth")]
pub async fn oauth(
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> Result<impl Responder, actix_web::Error> {
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
    let (authorize_url, csrf_token, nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        // This example is requesting access to the "calendar" features and the user's profile.
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    // add csrf token and nonce to state:
    state.csrf_token.lock().await.insert(
        serde_json::to_string(&csrf_token)?,
        serde_json::to_string(&nonce).unwrap_or_default(),
    );
    info!("...... csrftoken: {:?}", state.csrf_token.lock().await);
    Ok(Redirect::to(authorize_url.to_string()))
}
