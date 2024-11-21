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
    Consistent,
    Skipped { correct: String },
    DisparateAnswer { correct: String },
    GuessSubmitted(AnswerResponse),
    Unknown,
}

#[derive(Debug)]
pub struct ValidationResult {
    pub date: DoorDate,
    pub part1: Result<PartValidation>,
    pub part2: Result<PartValidation>,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum ValidationMode {
    #[default]
    Normal,
    DryRun,
}

pub async fn validate_answer(
    date: &DoorDate,
    answer: DoorResult,
    submitted: &DayResponse,
    mode: ValidationMode,
    client: &(impl AoCClient + Send + Sync),
) -> Result<ValidationResult> {
    use Part::*;
    Ok(ValidationResult {
        date: *date,
        part1: validate_part(
            date,
            Part1,
            answer.part1,
            submitted.first_half.as_deref(),
            mode,
            client,
        )
        .await?,
        part2: validate_part(
            date,
            Part2,
            answer.part2,
            submitted.second_half.as_deref(),
            mode,
            client,
        )
        .await?,
    })
}

async fn validate_part(
    date: &DoorDate,
    part: Part,
    guess: Result<DoorPartResult>,
    submitted: Option<&str>,
    mode: ValidationMode,
    client: &(impl AoCClient + Send + Sync),
) -> Result<Result<PartValidation>> {
    use PartValidity::*;

    Ok(match guess {
        Ok(DoorPartResult::Computed { ref answer, .. }) => {
            let validity = match (submitted, mode) {
                (Some(correct), _) if correct == answer => Consistent,
                (Some(correct), _) => DisparateAnswer {
                    correct: correct.to_owned(),
                },
                (None, ValidationMode::DryRun) => Unknown,
                (None, ValidationMode::Normal) => {
                    GuessSubmitted(client.post_answer(date, part, answer).await?)
                }
            };

            Ok(PartValidation {
                guess: guess.unwrap(),
                validity,
            })
        }
        Ok(DoorPartResult::Skipped) => Ok(PartValidation {
            guess: guess.unwrap(),
            validity: Skipped {
                correct: submitted.unwrap().to_owned(),
            },
        }),
        Err(e) => Err(e),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use assert_matches::assert_matches;

    use AnswerResponse::*;
    use PartValidity::*;

    const TEST_DAY: DoorDate = DoorDate {
        day: 17,
        year: 2042,
    };

    fn make_part_result(ans: &str) -> Result<DoorPartResult> {
        Ok(DoorPartResult::Computed {
            answer: ans.to_string(),
            time: Default::default(),
        })
    }

    struct FakeValidationClient {
        on_cooldown: bool,
    }

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
            let guess_num: i32 = guess.parse().unwrap();
            match (part, guess_num) {
                (Part1, 42) => Ok(Correct),
                (Part1, x) if x < 42 => Ok(IncorrectTooLow),
                (Part1, _) => Ok(IncorrectTooHigh),
                (Part2, 123) => Ok(Correct),
                (Part2, _) => Ok(IncorrectTooManyGuesses),
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
                ValidationMode::Normal,
                &client
            )
            .await,
            Ok(ValidationResult {
                date: TEST_DAY,
                part1: Ok(PartValidation {
                    validity: Consistent,
                    ..
                }),
                part2: Ok(PartValidation {
                    validity: Consistent,
                    ..
                })
            })
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
                ValidationMode::Normal,
                &client
            )
            .await,
            Ok(ValidationResult {
                date: TEST_DAY,
                part1: Ok(PartValidation { validity: DisparateAnswer { correct }, ..}),
                part2: Ok(PartValidation { validity: Consistent, ..})
            }) if &correct == "42"
        );
    }

    #[tokio::test]
    async fn answer_for_solved_part_is_still_validated_in_dry_run() {
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
                ValidationMode::DryRun,
                &client
            )
            .await,
            Ok(ValidationResult {
                date: TEST_DAY,
                part1: Ok(PartValidation { validity: DisparateAnswer { correct }, ..}),
                part2: Ok(PartValidation { validity: Consistent, ..})
            }) if &correct == "42"
        );
    }

    #[tokio::test]
    async fn correct_answer_for_unsolved_parts_is_submitted_in_normal_mode() {
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
                ValidationMode::Normal,
                &client
            )
            .await,
            Ok(ValidationResult {
                date: TEST_DAY,
                part1: Ok(PartValidation {
                    validity: GuessSubmitted(Correct),
                    ..
                }),
                part2: Ok(PartValidation {
                    validity: GuessSubmitted(Correct),
                    ..
                })
            })
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
                ValidationMode::Normal,
                &client
            )
            .await,
            Ok(ValidationResult {
                date: TEST_DAY,
                part1: Ok(PartValidation {
                    validity: Consistent,
                    ..
                }),
                part2: Ok(PartValidation {
                    validity: GuessSubmitted(Correct),
                    ..
                })
            })
        );
    }

    #[tokio::test]
    async fn answer_for_unsolved_parts_is_not_submitted_in_dry_run() {
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
                ValidationMode::DryRun,
                &client
            )
            .await,
            Ok(ValidationResult {
                date: TEST_DAY,
                part1: Ok(PartValidation {
                    validity: Unknown,
                    ..
                }),
                part2: Ok(PartValidation {
                    validity: Unknown,
                    ..
                })
            })
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
                ValidationMode::Normal,
                &client
            )
            .await,
            Ok(ValidationResult {
                date: TEST_DAY,
                part1: Ok(PartValidation {
                    validity: GuessSubmitted(IncorrectTooHigh { .. }),
                    ..
                }),
                part2: Ok(PartValidation {
                    validity: GuessSubmitted(IncorrectTooManyGuesses { .. }),
                    ..
                })
            })
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
                ValidationMode::Normal,
                &client
            )
            .await,
            Ok(ValidationResult {
                date: TEST_DAY,
                part1: Ok(PartValidation {
                    validity: Consistent,
                    ..
                }),
                part2: Ok(PartValidation {
                    validity: GuessSubmitted(GuessedTooRecently),
                    ..
                })
            })
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
                ValidationMode::Normal,
                &client
            )
            .await,
            Err(_)
        );
    }
}
