mod client;
mod response;

pub(crate) use client::AoCClient;
pub(crate) use response::{AnswerResponse, DayResponse};

#[allow(dead_code)]
pub fn plain_client() -> anyhow::Result<impl AoCClient> {
    client::WebClient::new()
}

#[allow(dead_code)]
pub fn caching_client(empty_cache: bool) -> anyhow::Result<impl AoCClient> {
    Ok(client::CachingClient::new(
        client::WebClient::new()?,
        if empty_cache {
            client::FilesystemCache::clean_tmp()?
        } else {
            client::FilesystemCache::tmp()
        },
    ))
}
