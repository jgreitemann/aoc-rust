use crate::api::{AnswerResponse, AoCClient, DayResponse};
use crate::door::{DoorDate, Part};

use anyhow::{anyhow, Context, Result};
use thiserror::Error;

use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Failed to retrieve SESSION environment variable")]
    CannotAccessSessionEnvVar,
    #[error("The session token is invalid; it may have expired. Log into https://adventofcode.com/ and update the session token.")]
    AuthenticationInvalidOrExpired,
}

pub struct WebClient {
    http: reqwest::Client,
}

impl WebClient {
    pub fn new() -> Result<Self> {
        let session = std::env::var("SESSION").context(SessionError::CannotAccessSessionEnvVar)?;

        let jar = reqwest::cookie::Jar::default();
        jar.add_cookie_str(
            &format!("session={session}"),
            &"https://adventofcode.com".parse().unwrap(),
        );

        let http = reqwest::Client::builder()
            .cookie_provider(Arc::new(jar))
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        Ok(Self { http })
    }
}

impl AoCClient for WebClient {
    async fn get_input(&self, &DoorDate { year, day }: &DoorDate) -> Result<String> {
        let resp = self
            .http
            .get(format!("https://adventofcode.com/{year}/day/{day}/input"))
            .send()
            .await?;

        use reqwest::StatusCode;
        match resp.status() {
            StatusCode::BAD_REQUEST => Err(SessionError::AuthenticationInvalidOrExpired)
                .with_context(|| {
                    format!("Failed to authenticate when downloading input for day {day}, {year}")
                }),
            StatusCode::OK => Ok(resp.text().await?),
            status => Err(anyhow!("{}", resp.text().await?)).with_context(|| {
                format!("Failed to download input for day {day}, {year}: HTTP {status}")
            }),
        }
    }

    async fn get_day(&self, &DoorDate { year, day }: &DoorDate) -> Result<DayResponse> {
        let resp = self
            .http
            .get(format!("https://adventofcode.com/{year}/day/{day}"))
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        Ok(DayResponse::parse(&resp))
    }

    async fn post_answer(
        &self,
        &DoorDate { year, day }: &DoorDate,
        part: Part,
        guess: &str,
    ) -> Result<AnswerResponse> {
        let part_string = part.to_string();
        let form = HashMap::from([("level", part_string.as_str()), ("answer", guess)]);
        let resp = self
            .http
            .post(format!("https://adventofcode.com/{year}/day/{day}/answer"))
            .form(&form)
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::from_u16(302).unwrap() {
            // Redirect in case of authentication failure
            return Err(SessionError::AuthenticationInvalidOrExpired).context(format!(
                    "Failed to authenticate when posting the answer {guess:?} for part {part} of day {day}, {year}"
            ));
        }

        let text = resp
            .error_for_status()
            .with_context(|| {
                format!("Failed to post answer {guess:?} for part {part} of day {day}, {year}")
            })?
            .text()
            .await?;

        AnswerResponse::parse(&text).with_context(|| {
            format!(
                    "Failed to parse server response after posting the answer {guess:?} for part {part} of day {day}, {year}"
            )
        })
    }
}
