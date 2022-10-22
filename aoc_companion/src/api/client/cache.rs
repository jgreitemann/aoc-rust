use crate::api::{AnswerResponse, AoCClient, DayResponse};
use crate::door::{DoorDate, Part};

use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Cache {
    async fn cache(&mut self, key: &str, value: &str);
    async fn recall(&self, key: &str) -> Option<String>;
    async fn dirty(&mut self, key: &str);
}

pub struct CachingClient<U, C>
where
    U: AoCClient + Send + Sync,
    C: Cache + Send + Sync,
{
    underlying_client: U,
    cache: tokio::sync::RwLock<C>,
}

impl<U, C> CachingClient<U, C>
where
    U: AoCClient + Send + Sync,
    C: Cache + Send + Sync,
{
    pub fn new(underlying_client: U, cache: C) -> Self {
        Self {
            underlying_client,
            cache: tokio::sync::RwLock::new(cache),
        }
    }

    fn input_key(&self, DoorDate { day, year }: &DoorDate) -> String {
        format!("input_{year}_{day}")
    }

    fn day_key(&self, DoorDate { day, year }: &DoorDate) -> String {
        format!("day_{year}_{day}")
    }
}

#[async_trait]
impl<U, C> AoCClient for CachingClient<U, C>
where
    U: AoCClient + Send + Sync,
    C: Cache + Send + Sync,
{
    async fn get_input(&self, date: &DoorDate) -> Result<String> {
        let key = self.input_key(date);
        let cache_result = self.cache.read().await.recall(&key).await;
        match cache_result {
            Some(cached) => Ok(cached),
            None => {
                let result = self.underlying_client.get_input(date).await;
                if let Ok(value) = &result {
                    self.cache.write().await.cache(&key, value).await;
                }
                result
            }
        }
    }

    async fn get_day(&self, date: &DoorDate) -> Result<DayResponse> {
        let key = self.day_key(date);
        let cache_result = self
            .cache
            .read()
            .await
            .recall(&key)
            .await
            .and_then(|s| serde_json::from_str(&s).ok());
        match cache_result {
            Some(cached) => Ok(cached),
            None => {
                let result = self.underlying_client.get_day(date).await;
                if let Some(value) = result
                    .as_ref()
                    .ok()
                    .and_then(|resp| serde_json::to_string(resp).ok())
                {
                    self.cache.write().await.cache(&key, &value).await;
                }
                result
            }
        }
    }

    async fn post_answer(
        &self,
        date: &DoorDate,
        part: Part,
        guess: &str,
    ) -> Result<AnswerResponse> {
        let result = self.underlying_client.post_answer(date, part, guess).await;
        if let Ok(AnswerResponse::Correct) = &result {
            self.cache.write().await.dirty(&self.day_key(date)).await;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use assert_matches::assert_matches;
    use itertools::{assert_equal, Itertools};
    use std::{collections::HashMap, sync::Mutex};

    use super::*;

    #[async_trait]
    impl Cache for HashMap<String, String> {
        async fn cache(&mut self, key: &str, value: &str) {
            self.insert(key.to_owned(), value.to_owned());
        }

        async fn recall(&self, key: &str) -> Option<String> {
            self.get(key).cloned()
        }

        async fn dirty(&mut self, key: &str) {
            self.remove(key);
        }
    }

    struct FakeUnderlyingClient {
        inputs_queried: Mutex<[bool; 25]>,
        days_queried: Mutex<[bool; 25]>,
        day_data: Mutex<[DayResponse; 25]>,
    }

    impl FakeUnderlyingClient {
        fn new() -> Self {
            Self {
                inputs_queried: Mutex::new([false; 25]),
                days_queried: Mutex::new([false; 25]),
                day_data: Mutex::new(<[DayResponse; 25]>::default()),
            }
        }
    }

    #[async_trait]
    impl AoCClient for FakeUnderlyingClient {
        async fn get_input(&self, date: &DoorDate) -> Result<String> {
            match date {
                DoorDate {
                    day: 1..=25,
                    year: 2042,
                } => {
                    let mut inputs = self.inputs_queried.lock().unwrap();
                    assert!(
                        !std::mem::replace(&mut inputs[(date.day - 1) as usize], true),
                        "Accessing input for {date:?} for the second time!"
                    );
                    Ok(format!("input {}", date.day))
                }
                _ => Err(anyhow!(
                    "Date not supported by fake; need to be in advent of 2042."
                )),
            }
        }

        async fn get_day(&self, date: &DoorDate) -> Result<DayResponse> {
            match date {
                DoorDate {
                    day: 1..=25,
                    year: 2042,
                } => {
                    let mut days = self.days_queried.lock().unwrap();
                    let index = (date.day - 1) as usize;
                    assert!(
                        !std::mem::replace(&mut days[index], true),
                        "Accessing day {date:?} for the second time without it being dirtied."
                    );
                    Ok(self.day_data.lock().unwrap()[index].clone())
                }
                _ => Err(anyhow!(
                    "Date not supported by fake; need to be in advent of 2042."
                )),
            }
        }

        async fn post_answer(
            &self,
            date: &DoorDate,
            part: Part,
            guess: &str,
        ) -> Result<AnswerResponse> {
            match date {
                DoorDate {
                    day: 1..=25,
                    year: 2042,
                } => {
                    use AnswerResponse::*;
                    match (part, guess) {
                        (Part::Part1, "42") | (Part::Part2, "17") => {
                            let index = (date.day - 1) as usize;
                            self.days_queried.lock().unwrap()[(date.day - 1) as usize] = false;
                            let mut data = self.day_data.lock().unwrap();
                            *match &part {
                                Part::Part1 => &mut data[index].first_half,
                                Part::Part2 => &mut data[index].second_half,
                            } = Some(format!("day {}, Part {part}", date.day));
                            Ok(Correct)
                        }
                        _ => Ok(IncorrectTooManyGuesses {
                            guess: guess.to_owned(),
                        }),
                    }
                }
                _ => Err(anyhow!(
                    "Date not supported by fake; need to be in advent of 2042."
                )),
            }
        }
    }

    const TEST_DATE_1: &DoorDate = &DoorDate {
        day: 17,
        year: 2042,
    };
    const TEST_DATE_2: &DoorDate = &DoorDate {
        day: 21,
        year: 2042,
    };

    #[tokio::test]
    async fn underlying_client_gets_input_when_not_in_cache() {
        let client = CachingClient::new(FakeUnderlyingClient::new(), HashMap::new());
        let input_1 = client.get_input(TEST_DATE_1).await.unwrap();
        let input_2 = client.get_input(TEST_DATE_2).await.unwrap();
        assert_equal(
            client.cache.read().await.values().sorted(),
            [&input_1, &input_2].into_iter().sorted(),
        );
    }

    #[tokio::test]
    async fn repeatedly_getting_input_for_the_same_day_only_uses_underlying_client_once() {
        let client = CachingClient::new(FakeUnderlyingClient::new(), HashMap::new());
        let input_1 = client.get_input(TEST_DATE_1).await.unwrap();

        // posting answer has no effect on input caching
        client
            .post_answer(TEST_DATE_1, Part::Part1, "42")
            .await
            .unwrap();

        let input_2 = client.get_input(TEST_DATE_1).await.unwrap();
        assert_eq!(input_1, input_2);
    }

    #[tokio::test]
    async fn underlying_client_gets_day_response_when_not_in_cache() {
        let client = CachingClient::new(FakeUnderlyingClient::new(), HashMap::new());
        client.get_day(TEST_DATE_1).await.unwrap();
        client.get_day(TEST_DATE_2).await.unwrap();
        assert_eq!(client.cache.read().await.len(), 2);
    }

    #[tokio::test]
    async fn after_posting_correct_answer_the_corresponding_day_is_no_longer_cached() {
        let client = CachingClient::new(FakeUnderlyingClient::new(), HashMap::new());
        client.get_day(TEST_DATE_1).await.unwrap();
        client.get_day(TEST_DATE_2).await.unwrap();
        assert_eq!(
            client
                .post_answer(TEST_DATE_1, Part::Part1, "42")
                .await
                .unwrap(),
            AnswerResponse::Correct
        );
        assert_eq!(client.cache.read().await.len(), 1);
        assert_eq!(
            client
                .post_answer(TEST_DATE_2, Part::Part2, "17")
                .await
                .unwrap(),
            AnswerResponse::Correct
        );
        assert!(client.cache.read().await.is_empty());
    }

    #[tokio::test]
    async fn repeatedly_getting_day_response_for_the_same_day_without_posting_answers_only_uses_underlying_client_once(
    ) {
        let client = CachingClient::new(FakeUnderlyingClient::new(), HashMap::new());
        let before = client.get_day(TEST_DATE_1).await.unwrap();
        let after = client.get_day(TEST_DATE_1).await.unwrap();
        assert_eq!(before, after);
    }

    #[tokio::test]
    async fn repeatedly_getting_day_response_for_the_same_day_uses_the_underlying_client_again_if_the_correct_answer_has_been_posted(
    ) {
        let client = CachingClient::new(FakeUnderlyingClient::new(), HashMap::new());
        let before = client.get_day(TEST_DATE_1).await.unwrap();
        assert_matches!(
            before,
            DayResponse {
                first_half: None,
                second_half: None
            }
        );
        client
            .post_answer(TEST_DATE_1, Part::Part1, "42")
            .await
            .unwrap();
        let after = client.get_day(TEST_DATE_1).await.unwrap();
        assert_matches!(
            after,
            DayResponse {
                first_half: Some(_),
                second_half: None
            }
        );
    }

    #[tokio::test]
    async fn repeatedly_getting_day_response_for_the_same_day_only_uses_the_underlying_client_once_if_an_incorrect_answer_has_been_posted(
    ) {
        let client = CachingClient::new(FakeUnderlyingClient::new(), HashMap::new());
        let before = client.get_day(TEST_DATE_1).await.unwrap();
        client
            .post_answer(TEST_DATE_1, Part::Part1, "41")
            .await
            .unwrap();
        let after = client.get_day(TEST_DATE_1).await.unwrap();
        assert_eq!(before, after);
    }
}
