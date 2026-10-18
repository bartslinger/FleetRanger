//! The OpenID Connect authorization code flow with PKCE.
//!
//! The provider's metadata and signing keys are fetched for every login. Logins
//! are rare, and this way rotated keys are never stale.

use std::time::Duration;

use anyhow::{Context, Result};
use openidconnect::core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata};
use openidconnect::url::{Url, form_urlencoded};
use openidconnect::{
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenResponse,
};

use crate::config::OidcConfig;

pub struct Provider {
    issuer: IssuerUrl,
    client_id: ClientId,
    client_secret: ClientSecret,
    redirect_url: RedirectUrl,
    http: openidconnect::ureq::Agent,
}

/// What the browser keeps, in a short-lived cookie, between leaving for the
/// provider and coming back.
pub struct PendingLogin {
    state: String,
    nonce: String,
    pkce_verifier: String,
    /// The page to show after logging in.
    pub return_to: String,
}

/// The person the provider vouches for.
pub struct Identity {
    pub subject: String,
    /// Only set if the provider has verified it.
    pub verified_email: Option<String>,
    pub name: Option<String>,
}

impl Provider {
    pub fn new(config: &OidcConfig, public_url: &Url) -> Result<Self> {
        let redirect_url = public_url
            .join("/auth/callback")
            .context("invalid public URL")?;
        Ok(Self {
            issuer: IssuerUrl::new(config.issuer.clone()).context("OIDC_ISSUER is not a URL")?,
            client_id: ClientId::new(config.client_id.clone()),
            client_secret: ClientSecret::new(config.client_secret.clone()),
            redirect_url: RedirectUrl::from_url(redirect_url),
            http: openidconnect::ureq::AgentBuilder::new()
                // Following redirects would open the client up to request forgery.
                .redirects(0)
                .timeout(Duration::from_secs(10))
                .build(),
        })
    }

    /// The provider's login page to send the browser to. Blocks on HTTP.
    pub fn begin(&self, return_to: String) -> Result<(Url, PendingLogin)> {
        let client = self.client()?;
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (url, state, nonce) = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("email".to_owned()))
            .add_scope(Scope::new("profile".to_owned()))
            .set_pkce_challenge(pkce_challenge)
            .url();
        let pending = PendingLogin {
            state: state.into_secret(),
            nonce: nonce.secret().clone(),
            pkce_verifier: pkce_verifier.into_secret(),
            return_to,
        };
        Ok((url, pending))
    }

    /// Checks the provider's answer and returns who logged in. Blocks on HTTP.
    pub fn finish(&self, pending: PendingLogin, state: &str, code: String) -> Result<Identity> {
        anyhow::ensure!(state == pending.state, "state does not match");
        let client = self.client()?;
        let response = client
            .exchange_code(AuthorizationCode::new(code))?
            .set_pkce_verifier(PkceCodeVerifier::new(pending.pkce_verifier))
            .request(&self.http)
            .context("exchanging the authorization code")?;
        let id_token = response.id_token().context("no ID token in the response")?;
        let claims = id_token
            .claims(&client.id_token_verifier(), &Nonce::new(pending.nonce))
            .context("verifying the ID token")?;
        Ok(Identity {
            subject: claims.subject().to_string(),
            verified_email: claims
                .email()
                .filter(|_| claims.email_verified() == Some(true))
                .map(|email| email.to_string()),
            name: claims
                .name()
                .and_then(|name| name.get(None))
                .map(|name| name.to_string()),
        })
    }

    fn client(
        &self,
    ) -> Result<
        CoreClient<
            openidconnect::EndpointSet,
            openidconnect::EndpointNotSet,
            openidconnect::EndpointNotSet,
            openidconnect::EndpointNotSet,
            openidconnect::EndpointMaybeSet,
            openidconnect::EndpointMaybeSet,
        >,
    > {
        let metadata = CoreProviderMetadata::discover(&self.issuer, &self.http)
            .context("fetching the login provider's metadata")?;
        Ok(CoreClient::from_provider_metadata(
            metadata,
            self.client_id.clone(),
            Some(self.client_secret.clone()),
        )
        .set_redirect_uri(self.redirect_url.clone()))
    }
}

impl PendingLogin {
    /// As a cookie value: URL-encoded form fields hold only cookie-safe characters.
    pub fn encode(&self) -> String {
        form_urlencoded::Serializer::new(String::new())
            .append_pair("state", &self.state)
            .append_pair("nonce", &self.nonce)
            .append_pair("pkce_verifier", &self.pkce_verifier)
            .append_pair("return_to", &self.return_to)
            .finish()
    }

    pub fn decode(value: &str) -> Option<Self> {
        let (mut state, mut nonce, mut pkce_verifier, mut return_to) = (None, None, None, None);
        for (key, value) in form_urlencoded::parse(value.as_bytes()) {
            let field = match &*key {
                "state" => &mut state,
                "nonce" => &mut nonce,
                "pkce_verifier" => &mut pkce_verifier,
                "return_to" => &mut return_to,
                _ => continue,
            };
            *field = Some(value.into_owned());
        }
        Some(Self {
            state: state?,
            nonce: nonce?,
            pkce_verifier: pkce_verifier?,
            return_to: return_to?,
        })
    }
}
