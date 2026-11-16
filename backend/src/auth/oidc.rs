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
use crate::organisations::signup;

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
    /// What was chosen on the sign-up page, when signing up. Anyone can set
    /// it in their own cookie, but it only names the organisation they create.
    pub signup: Option<signup::Request>,
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
    pub fn begin(
        &self,
        return_to: String,
        signup: Option<signup::Request>,
    ) -> Result<(Url, PendingLogin)> {
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
            signup,
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
        let mut serializer = form_urlencoded::Serializer::new(String::new());
        serializer
            .append_pair("state", &self.state)
            .append_pair("nonce", &self.nonce)
            .append_pair("pkce_verifier", &self.pkce_verifier)
            .append_pair("return_to", &self.return_to);
        if let Some(signup) = &self.signup {
            serializer
                .append_pair("organisation_name", &signup.organisation_name)
                .append_pair("time_zone", &signup.time_zone);
        }
        serializer.finish()
    }

    pub fn decode(value: &str) -> Option<Self> {
        let (mut state, mut nonce, mut pkce_verifier, mut return_to) = (None, None, None, None);
        let (mut organisation_name, mut time_zone) = (None, None);
        for (key, value) in form_urlencoded::parse(value.as_bytes()) {
            let field = match &*key {
                "state" => &mut state,
                "nonce" => &mut nonce,
                "pkce_verifier" => &mut pkce_verifier,
                "return_to" => &mut return_to,
                "organisation_name" => &mut organisation_name,
                "time_zone" => &mut time_zone,
                _ => continue,
            };
            *field = Some(value.into_owned());
        }
        Some(Self {
            state: state?,
            nonce: nonce?,
            pkce_verifier: pkce_verifier?,
            return_to: return_to?,
            signup: organisation_name
                .zip(time_zone)
                .map(|(organisation_name, time_zone)| signup::Request {
                    organisation_name,
                    time_zone,
                }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::PendingLogin;
    use crate::organisations::signup::Request;

    /// The sign-up choices travel in the login cookie, hand-encoded: a name
    /// with the encoding's own characters or accents must come back intact,
    /// and half a pair means no sign-up at all.
    #[test]
    fn sign_up_choices_survive_the_cookie() {
        let request = Request {
            organisation_name: "Test & Co = 1; Ålesund".to_owned(),
            time_zone: "Europe/Oslo".to_owned(),
        };
        let pending = PendingLogin {
            state: "state".to_owned(),
            nonce: "nonce".to_owned(),
            pkce_verifier: "verifier".to_owned(),
            return_to: "/".to_owned(),
            signup: Some(request.clone()),
        };
        let back = PendingLogin::decode(&pending.encode()).unwrap();
        assert_eq!(back.signup, Some(request.clone()));
        assert_eq!(Request::decode(&request.encode()), Some(request));

        let half = "state=s&nonce=n&pkce_verifier=v&return_to=%2F&organisation_name=Test";
        assert_eq!(PendingLogin::decode(half).unwrap().signup, None);
        assert_eq!(Request::decode("organisation_name=Test"), None);
    }
}
