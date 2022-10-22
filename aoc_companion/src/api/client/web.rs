use crate::api::{AnswerResponse, AoCClient, DayResponse};
use crate::door::{DoorDate, Part};

use anyhow::{Context, Result};
use async_trait::async_trait;

use std::collections::HashMap;
use std::sync::Arc;

pub struct WebClient {
    http: reqwest::Client,
}

impl WebClient {
    pub fn new() -> Result<Self> {
        let session =
            std::env::var("SESSION").context("Failed to retrieve SESSION environment variable")?;

        let jar = reqwest::cookie::Jar::default();
        jar.add_cookie_str(
            &format!("session={session}"),
            &"https://adventofcode.com".parse().unwrap(),
        );

        let http = reqwest::Client::builder()
            .cookie_provider(Arc::new(jar))
            .build()?;

        Ok(Self { http })
    }
}

#[async_trait]
impl AoCClient for WebClient {
    async fn get_input(&self, &DoorDate { year, day }: &DoorDate) -> Result<String> {
        let resp = self
            .http
            .get(format!("https://adventofcode.com/{year}/day/{day}/input"))
            .send()
            .await?
            .text()
            .await?;
        Ok(resp)
    }

    async fn get_day(&self, &DoorDate { year, day }: &DoorDate) -> Result<DayResponse> {
        let resp = self
            .http
            .get(format!("https://adventofcode.com/{year}/day/{day}"))
            .send()
            .await?
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
            .await?
            .text()
            .await?;
        Ok(AnswerResponse::parse(&resp))
    }
}
