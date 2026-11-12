//! Requests log decryption keys from DJI. The parser builds the request, but
//! FleetRanger sends it itself so that exactly what was sent can be stored.

use std::time::Duration;

use anyhow::{Context, Result};
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

pub type Keychains = Vec<Vec<KeychainFeaturePoint>>;

/// What an exchange means for the log it was for.
#[derive(Debug)]
pub enum Reply {
    /// DJI answered about the log: its keys, or why there are none (the
    /// log's fault).
    Answered(std::result::Result<Keychains, String>),
    /// No answer about the log: DJI unreachable, refusing the app key,
    /// rate limiting, in trouble, or an answer that broke off or isn't
    /// DJI's. The log waits.
    Down(String),
}

pub fn reply(exchange: &Exchange) -> Reply {
    let error = exchange.error.as_deref().unwrap_or_default();
    match (exchange.status, &exchange.body) {
        (None, _) => Reply::Down(format!("DJI did not answer: {error}")),
        (Some(status), None) => Reply::Down(format!("DJI answered {status}, but {error}")),
        (Some(status @ (401 | 403 | 429 | 500..)), _) => {
            Reply::Down(format!("DJI answered {status}"))
        }
        (Some(200), Some(body)) => match keychains(body) {
            Ok(answer) => Reply::Answered(answer),
            Err(error) => Reply::Down(format!("{error:#}")),
        },
        (Some(status), Some(_)) => Reply::Answered(Err(format!("DJI answered {status}"))),
    }
}

/// The keychains in a response body with status 200, or why DJI gave none;
/// an error if the body isn't a response of DJI's.
pub fn keychains(body: &str) -> Result<std::result::Result<Keychains, String>> {
    let response: KeychainsResponse =
        serde_json::from_str(body).context("DJI's response is not what the parser expects")?;
    if response.result.code != 0 {
        return Ok(Err(format!(
            "DJI refused the request: {} (code {})",
            response.result.msg, response.result.code
        )));
    }
    Ok(response
        .data
        .ok_or_else(|| "DJI's response contains no keys".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::{Exchange, Reply, reply};

    fn exchange(status: Option<u16>, body: Option<&str>) -> Reply {
        reply(&Exchange {
            status,
            body: body.map(str::to_owned),
            error: body.is_none().then(|| "Test error".to_owned()),
        })
    }

    /// Only an answer about the log itself fails it; trouble on the way or
    /// at DJI leaves it waiting, also when DJI's status was 200.
    #[test]
    fn only_answers_about_the_log_fail_it() {
        let keys = r#"{"data": [], "result": {"code": 0, "msg": "ok"}}"#;
        let refused = r#"{"data": null, "result": {"code": 3, "msg": "Test refusal"}}"#;
        assert!(matches!(
            exchange(Some(200), Some(keys)),
            Reply::Answered(Ok(_))
        ));
        assert!(matches!(
            exchange(Some(200), Some(refused)),
            Reply::Answered(Err(_))
        ));
        assert!(matches!(
            exchange(Some(400), Some("")),
            Reply::Answered(Err(_))
        ));
        for (status, body) in [
            (None, None),
            (Some(200), None),
            (Some(200), Some("<html>Test proxy</html>")),
            (Some(429), Some("")),
            (Some(503), Some("")),
        ] {
            assert!(
                matches!(exchange(status, body), Reply::Down(_)),
                "{status:?} {body:?}"
            );
        }
    }
}
