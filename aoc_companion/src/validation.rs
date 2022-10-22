use crate::api::*;
use crate::door::{DoorDate, DoorPartResult, DoorResult, Part};

use anyhow::Result;

#[derive(Debug, PartialEq)]
pub struct PartValidation {
    pub guess: DoorPartResult,
    pub validity: PartValidity,
}

#[derive(Debug, PartialEq)]
pub enum PartValidity {
    AlreadySolved,
    DisparateAnswer { correct: String },
    GuessSubmitted(AnswerResponse),
}

#[derive(Debug)]
pub struct ValidationResult {
    pub date: DoorDate,
    pub part1: Result<PartValidation>,
    pub part2: Result<PartValidation>,
}

pub async fn validate_answer(
    date: &DoorDate,
    answer: DoorResult,
    submitted: &DayResponse,
    client: &(dyn AoCClient + Send + Sync),
) -> ValidationResult {
    use Part::*;
    ValidationResult {
        date: date.clone(),
        part1: validate_part(
            date,
            Part1,
            answer.part1,
            submitted.first_half.as_deref(),
            client,
        )
        .await,
        part2: validate_part(
            date,
            Part2,
            answer.part2,
            submitted.second_half.as_deref(),
            client,
        )
        .await,
    }
}

async fn validate_part(
    date: &DoorDate,
    part: Part,
    guess: Result<DoorPartResult>,
    submitted: Option<&str>,
    client: &(dyn AoCClient + Send + Sync),
) -> Result<PartValidation> {
    use PartValidity::*;
    let guess = guess?;
    let answer = &guess.answer;
    let validity = match submitted {
        Some(correct) if correct == answer => AlreadySolved,
        Some(correct) => DisparateAnswer {
            correct: correct.to_owned(),
        },
        None => GuessSubmitted(client.post_answer(date, part, &answer).await?),
    };

    // sanity check

    use AnswerResponse::*;
    match &validity {
        GuessSubmitted(IncorrectTooHigh { guess })
        | GuessSubmitted(IncorrectTooLow { guess })
        | GuessSubmitted(IncorrectTooManyGuesses { guess }) => {
            assert_eq!(guess, answer, "The guess '{guess}' stated in the server response does not match the answer '{answer}' which we calculated; this may indicate a bug which led to us submitting the wrong thing");
        }
        _ => {}
    }

    Ok(PartValidation { guess, validity })
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use assert_matches::assert_matches;
    use async_trait::async_trait;

    use AnswerResponse::*;
    use PartValidity::*;

    const TEST_DAY: DoorDate = DoorDate {
        day: 17,
        year: 2042,
    };

    fn make_part_result(ans: &str) -> Result<DoorPartResult> {
        Ok(DoorPartResult {
            answer: ans.to_string(),
            time: Default::default(),
        })
    }

    struct FakeValidationClient {
        on_cooldown: bool,
    }

    #[async_trait]
    impl AoCClient for FakeValidationClient {
        async fn get_input(&self, _: &DoorDate) -> Result<String> {
            panic!("operation not supported by fake")
        }

        async fn get_day(&self, _: &DoorDate) -> Result<DayResponse> {
            panic!("operation not supported by fake")
        }

        async fn post_answer(
            &self,
            date: &DoorDate,
            part: Part,
            guess: &str,
        ) -> Result<AnswerResponse> {
            use AnswerResponse::*;
            use Part::*;

            if self.on_cooldown {
                return Ok(GuessedTooRecently);
            }

            if date.day > 25 {
                return Err(anyhow!("Christmas is over!"));
            }

            assert_eq!(date, &TEST_DAY);
            let guess = guess.to_owned();
            let guess_num: i32 = guess.parse().unwrap();
            match (part, guess_num) {
                (Part1, 42) => Ok(Correct),
                (Part1, x) if x < 42 => Ok(IncorrectTooLow { guess }),
                (Part1, _) => Ok(IncorrectTooHigh { guess }),
                (Part2, 123) => Ok(Correct),
                (Part2, _) => Ok(IncorrectTooManyGuesses { guess }),
            }
        }
    }

    #[tokio::test]
    async fn correct_answer_for_solved_part() {
        let client = FakeValidationClient { on_cooldown: true };
        assert_matches!(
            validate_answer(
                &TEST_DAY,
                DoorResult {
                    part1: make_part_result("42"),
                    part2: make_part_result("123")
                },
                &DayResponse {
                    first_half: Some("42".to_string()),
                    second_half: Some("123".to_string())
                },
                &client
            )
            .await,
            ValidationResult {
                date: TEST_DAY,
                part1: Ok(PartValidation {
                    validity: AlreadySolved,
                    ..
                }),
                part2: Ok(PartValidation {
                    validity: AlreadySolved,
                    ..
                })
            }
        );
    }

    #[tokio::test]
    async fn incorrect_answer_for_solved_parts_yields_disparity() {
        let client = FakeValidationClient { on_cooldown: true };
        assert_matches!(
            validate_answer(
                &TEST_DAY,
                DoorResult {
                    part1: make_part_result("43"),
                    part2: make_part_result("123")
                },
                &DayResponse {
                    first_half: Some("42".to_string()),
                    second_half: Some("123".to_string())
                },
                &client
            )
            .await,
            ValidationResult {
                date: TEST_DAY,
                part1: Ok(PartValidation { validity: DisparateAnswer { correct }, ..}),
                part2: Ok(PartValidation { validity: AlreadySolved, ..})
            } if &correct == "42"
        );
    }

    #[tokio::test]
    async fn correct_answer_for_unsolved_parts_is_submitted() {
        let client = FakeValidationClient { on_cooldown: false };
        assert_matches!(
            validate_answer(
                &TEST_DAY,
                DoorResult {
                    part1: make_part_result("42"),
                    part2: make_part_result("123")
                },
                &DayResponse {
                    first_half: None,
                    second_half: None
                },
                &client
            )
            .await,
            ValidationResult {
                date: TEST_DAY,
                part1: Ok(PartValidation {
                    validity: GuessSubmitted(Correct),
                    ..
                }),
                part2: Ok(PartValidation {
                    validity: GuessSubmitted(Correct),
                    ..
                })
            }
        );
        assert_matches!(
            validate_answer(
                &TEST_DAY,
                DoorResult {
                    part1: make_part_result("42"),
                    part2: make_part_result("123")
                },
                &DayResponse {
                    first_half: Some("42".to_string()),
                    second_half: None
                },
                &client
            )
            .await,
            ValidationResult {
                date: TEST_DAY,
                part1: Ok(PartValidation {
                    validity: AlreadySolved,
                    ..
                }),
                part2: Ok(PartValidation {
                    validity: GuessSubmitted(Correct),
                    ..
                })
            }
        );
    }

    #[tokio::test]
    async fn incorrect_answer_for_unsolved_parts() {
        let client = FakeValidationClient { on_cooldown: false };
        assert_matches!(
            validate_answer(
                &TEST_DAY,
                DoorResult {
                    part1: make_part_result("43"),
                    part2: make_part_result("122")
                },
                &DayResponse {
                    first_half: None,
                    second_half: None
                },
                &client
            )
            .await,
            ValidationResult {
                date: TEST_DAY,
                part1: Ok(PartValidation {
                    validity: GuessSubmitted(IncorrectTooHigh { .. }),
                    ..
                }),
                part2: Ok(PartValidation {
                    validity: GuessSubmitted(IncorrectTooManyGuesses { .. }),
                    ..
                })
            }
        );
    }

    #[tokio::test]
    async fn submitting_answer_during_cooldown() {
        let client = FakeValidationClient { on_cooldown: true };
        assert_matches!(
            validate_answer(
                &TEST_DAY,
                DoorResult {
                    part1: make_part_result("42"),
                    part2: make_part_result("123")
                },
                &DayResponse {
                    first_half: Some("42".to_string()),
                    second_half: None
                },
                &client
            )
            .await,
            ValidationResult {
                date: TEST_DAY,
                part1: Ok(PartValidation {
                    validity: AlreadySolved,
                    ..
                }),
                part2: Ok(PartValidation {
                    validity: GuessSubmitted(GuessedTooRecently),
                    ..
                })
            }
        );
    }

    #[tokio::test]
    async fn client_errors_are_propagated_when_answer_failed_to_submit() {
        let client = FakeValidationClient { on_cooldown: false };
        assert_matches!(
            validate_answer(
                &DoorDate {
                    day: 27,
                    year: 2042
                },
                DoorResult {
                    part1: make_part_result("42"),
                    part2: make_part_result("123")
                },
                &DayResponse {
                    first_half: Some("42".to_string()),
                    second_half: None
                },
                &client
            )
            .await,
            ValidationResult {
                date: DoorDate {
                    day: 27,
                    year: 2042
                },
                part1: Ok(PartValidation {
                    validity: AlreadySolved,
                    ..
                }),
                part2: Err(_)
            }
        );
    }
}
