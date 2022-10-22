use itertools::Itertools;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct DayResponse {
    pub first_half: Option<String>,
    pub second_half: Option<String>,
}

fn solution_after_article(article: ElementRef) -> Option<String> {
    let code_selector = Selector::parse("code").unwrap();
    let after_article = article
        .next_siblings()
        .flat_map(|node| ElementRef::wrap(node))
        .next()
        .unwrap();
    if after_article.inner_html().contains("puzzle answer") {
        after_article
            .select(&code_selector)
            .next()
            .map(|code| code.inner_html())
    } else {
        None
    }
}

impl DayResponse {
    pub fn parse(response: &str) -> Self {
        let document = Html::parse_document(response);
        let article_selector = Selector::parse("article").unwrap();
        let articles = document.select(&article_selector);

        let (first_half, second_half) = articles
            .map(solution_after_article)
            .chain(std::iter::repeat(None))
            .next_tuple()
            .unwrap();

        Self {
            first_half,
            second_half,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AnswerResponse {
    IncorrectTooLow { guess: String },
    IncorrectTooHigh { guess: String },
    IncorrectTooManyGuesses { guess: String },
    GuessedTooRecently,
    Correct,
}

impl AnswerResponse {
    pub fn parse(response: &str) -> Self {
        let document = Html::parse_document(response);
        let article_selector = Selector::parse("article").unwrap();
        let p_selector = Selector::parse("p").unwrap();
        let code_selector = Selector::parse("code").unwrap();
        let article = document.select(&article_selector).next().unwrap();
        let paragraph = article.select(&p_selector).next().unwrap();
        let contains_text = |needle: &str| paragraph.text().any(|text| text.contains(needle));

        if contains_text("You gave an answer too recently") {
            Self::GuessedTooRecently
        } else {
            if contains_text("not the right answer") {
                let guess = paragraph
                    .select(&code_selector)
                    .next()
                    .unwrap()
                    .inner_html();

                if contains_text("too low") {
                    Self::IncorrectTooLow { guess }
                } else if contains_text("too high") {
                    Self::IncorrectTooHigh { guess }
                } else {
                    Self::IncorrectTooManyGuesses { guess }
                }
            } else if contains_text("That's the right answer") {
                Self::Correct
            } else {
                panic!("Unexpected response!")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    const UNSOLVED_DAY: &'static str = include_str!("data/day/unsolved.html");
    const ONELINE_INPUT_UNSOLVED_DAY: &'static str =
        include_str!("data/day/unsolved_with_oneline_input.html");
    const PARTIALLY_SOLVED_DAY: &'static str = include_str!("data/day/partially_solved.html");
    const ONELINE_INPUT_PARTIALLY_SOLVED_DAY: &'static str =
        include_str!("data/day/partially_solved_with_oneline_input.html");
    const FULLY_SOLVED_DAY: &'static str = include_str!("data/day/fully_solved.html");
    const ONELINE_INPUT_FULLY_SOLVED_DAY: &'static str =
        include_str!("data/day/fully_solved_with_oneline_input.html");

    #[test]
    fn day_response_determines_solved_puzzle_halves() {
        assert_matches!(
            DayResponse::parse(UNSOLVED_DAY),
            DayResponse {
                first_half: None,
                second_half: None,
            }
        );
        assert_matches!(
            DayResponse::parse(PARTIALLY_SOLVED_DAY),
            DayResponse {
                first_half: Some(_),
                second_half: None,
            }
        );
        assert_matches!(
            DayResponse::parse(FULLY_SOLVED_DAY),
            DayResponse {
                first_half: Some(_),
                second_half: Some(_),
            }
        );
    }

    #[test]
    fn day_response_determines_solved_puzzle_halves_for_days_with_oneline_input() {
        assert_matches!(
            DayResponse::parse(ONELINE_INPUT_UNSOLVED_DAY),
            DayResponse {
                first_half: None,
                second_half: None,
            }
        );
        assert_matches!(
            DayResponse::parse(ONELINE_INPUT_PARTIALLY_SOLVED_DAY),
            DayResponse {
                first_half: Some(_),
                second_half: None,
            }
        );
        assert_matches!(
            DayResponse::parse(ONELINE_INPUT_FULLY_SOLVED_DAY),
            DayResponse {
                first_half: Some(_),
                second_half: Some(_),
            }
        );
    }

    #[test]
    fn day_response_determines_solutions() {
        assert_eq!(
            DayResponse::parse(PARTIALLY_SOLVED_DAY),
            DayResponse {
                first_half: Some("893700".to_string()),
                second_half: None,
            }
        );
        assert_eq!(
            DayResponse::parse(FULLY_SOLVED_DAY),
            DayResponse {
                first_half: Some("392043".to_string()),
                second_half: Some("1605968119".to_string()),
            }
        );
    }

    #[test]
    fn day_response_determines_solutions_for_days_with_oneline_input() {
        assert_eq!(
            DayResponse::parse(ONELINE_INPUT_PARTIALLY_SOLVED_DAY),
            DayResponse {
                first_half: Some("438".to_string()),
                second_half: None,
            }
        );
        assert_eq!(
            DayResponse::parse(ONELINE_INPUT_FULLY_SOLVED_DAY),
            DayResponse {
                first_half: Some("438".to_string()),
                second_half: Some("266330".to_string()),
            }
        );
    }

    const ANSWER_CORRECT_PART_1: &str = include_str!("data/answer/correct_part1.html");
    const ANSWER_CORRECT_PART_2: &str = include_str!("data/answer/correct_part2.html");
    const ANSWER_TOO_LOW: &str = include_str!("data/answer/incorrect_too_low.html");
    const ANSWER_TOO_HIGH: &str = include_str!("data/answer/incorrect_too_high.html");
    const ANSWER_INCORRECT_AFTER_MANY_GUESSES: &str =
        include_str!("data/answer/incorrect_too_many_guesses.html");
    const ANSWER_GUESSED_TOO_RECENTLY: &str = include_str!("data/answer/guessed_too_recently.html");

    #[test]
    fn answer_response_determines_solution_correctness() {
        assert_matches!(
            AnswerResponse::parse(&ANSWER_CORRECT_PART_1),
            AnswerResponse::Correct
        );
        assert_matches!(
            AnswerResponse::parse(&ANSWER_CORRECT_PART_2),
            AnswerResponse::Correct
        );
        assert_matches!(
            AnswerResponse::parse(&ANSWER_TOO_LOW),
            AnswerResponse::IncorrectTooLow { .. }
        );
        assert_matches!(
            AnswerResponse::parse(&ANSWER_TOO_HIGH),
            AnswerResponse::IncorrectTooHigh { .. }
        );
        assert_matches!(
            AnswerResponse::parse(&ANSWER_INCORRECT_AFTER_MANY_GUESSES),
            AnswerResponse::IncorrectTooManyGuesses { .. }
        );
        assert_matches!(
            AnswerResponse::parse(&ANSWER_GUESSED_TOO_RECENTLY),
            AnswerResponse::GuessedTooRecently
        );
    }

    #[test]
    fn answer_response_determines_guess() {
        assert_eq!(
            AnswerResponse::parse(&ANSWER_TOO_LOW),
            AnswerResponse::IncorrectTooLow {
                guess: "234234".to_string()
            }
        );
        assert_eq!(
            AnswerResponse::parse(&ANSWER_TOO_HIGH),
            AnswerResponse::IncorrectTooHigh {
                guess: "985639847539754389578395".to_string()
            }
        );
        assert_eq!(
            AnswerResponse::parse(&ANSWER_INCORRECT_AFTER_MANY_GUESSES),
            AnswerResponse::IncorrectTooManyGuesses {
                guess: "435".to_string()
            }
        );
    }
}
