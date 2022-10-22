mod client;
mod response;

pub(crate) use client::AoCClient;
pub(crate) use response::{AnswerResponse, DayResponse};

#[allow(dead_code)]
pub fn plain_client() -> anyhow::Result<impl AoCClient> {
    client::WebClient::new()
}

#[allow(dead_code)]
pub fn caching_client() -> anyhow::Result<impl AoCClient> {
    Ok(client::CachingClient::new(
        client::WebClient::new()?,
        client::FilesystemCache::tmp(),
    ))
}
