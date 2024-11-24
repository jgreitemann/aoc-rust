mod cache;
mod fs;
mod web;

use crate::api::{AnswerResponse, DayResponse};
use crate::door::{DoorDate, Part};

use anyhow::Result;

pub trait AoCClient {
    async fn get_input(&self, date: &DoorDate) -> Result<String>;
    async fn get_day(&self, date: &DoorDate) -> Result<DayResponse>;
    async fn post_answer(&self, date: &DoorDate, part: Part, guess: &str)
        -> Result<AnswerResponse>;
}

pub use cache::{Cache, CachingClient};
pub use fs::FilesystemCache;
pub use web::WebClient;
