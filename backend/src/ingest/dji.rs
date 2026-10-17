//! Requests log decryption keys from DJI. The parser builds the request, but
//! FleetRanger sends it itself so that exactly what was sent can be stored.

use std::time::Duration;

use anyhow::{Context, Result, bail};
use dji_log_parser::keychain::{KeychainFeaturePoint, KeychainsResponse};

const ENDPOINT: &str = "https://dev.dji.com/openapi/v1/flight-records/keychains";

/// What came back from DJI. `status` is `None` if DJI never answered.
pub struct Exchange {
    pub status: Option<u16>,
    pub body: Option<String>,
    pub error: Option<String>,
}

/// Sends a keychains request. Blocks until DJI answers or the request times out.
pub fn send(api_key: &str, request_body: &str) -> Exchange {
    let response = ureq::post(ENDPOINT)
        .set("Content-Type", "application/json")
        .set("Api-Key", api_key)
        .timeout(Duration::from_secs(30))
        .send_string(request_body);
    let response = match response {
        Ok(response) | Err(ureq::Error::Status(_, response)) => response,
        Err(error) => {
            return Exchange {
                status: None,
                body: None,
                error: Some(error.to_string()),
            };
        }
    };

    let status = response.status();
    match response.into_string() {
        Ok(body) => Exchange {
            status: Some(status),
            body: Some(body),
            error: None,
        },
        Err(error) => Exchange {
            status: Some(status),
            body: None,
            error: Some(format!("failed to read the response: {error}")),
        },
    }
}

/// The keychains in a successful response body.
pub fn keychains(body: &str) -> Result<Vec<Vec<KeychainFeaturePoint>>> {
    let response: KeychainsResponse =
        serde_json::from_str(body).context("DJI's response is not what the parser expects")?;
    if response.result.code != 0 {
        bail!(
            "DJI refused the request: {} (code {})",
            response.result.msg,
            response.result.code
        );
    }
    response.data.context("DJI's response contains no keys")
}
