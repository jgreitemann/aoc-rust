use crate::api::AnswerResponse;
use crate::door::*;
use crate::validation::*;

use anyhow::Result;
use tokio::sync::mpsc;

use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::io::Write;

#[derive(Debug)]
pub enum Progress {
    Pending,
    DownloadingInput,
    DownloadingDay,
    ComputingAnswer,
    ValidatingAnswer,
    Complete(ValidationResult),
}

#[derive(Debug)]
pub struct DoorProgress(pub DoorDate, pub Progress);

pub struct Table(BTreeMap<DoorDate, Progress>);

fn write_answer(
    f: &mut Formatter,
    DoorDate { day, year }: &DoorDate,
    part: Part,
    validation: &Result<PartValidation>,
) -> std::fmt::Result {
    use AnswerResponse::*;
    use PartValidity::*;
    let (message, emoji) = match validation {
        Ok(PartValidation {
            guess: DoorPartResult::Computed { answer, time },
            validity: Consistent,
        }) => (format!("{answer} ({time:?})"), '⭐'),
        Ok(PartValidation {
            guess: DoorPartResult::Computed { answer, .. },
            validity: DisparateAnswer { correct },
        }) => (format!("{answer}, but correct answer was {correct}"), '💢'),
        Ok(PartValidation {
            guess: DoorPartResult::Computed { answer, time },
            validity: GuessSubmitted(Correct),
        }) => (format!("{answer} ({time:?})"), '🌟'),
        Ok(PartValidation {
            guess: DoorPartResult::Computed { answer, .. },
            validity: GuessSubmitted(GuessedTooRecently),
        }) => (
            format!("{answer} (unable to submit; guessed too recently)"),
            '🕑',
        ),
        Ok(PartValidation {
            guess: DoorPartResult::Skipped,
            validity: Skipped { correct },
        }) => (format!("{correct} (skipped)"), '⭐'),
        Ok(PartValidation {
            guess: DoorPartResult::Skipped,
            validity: _,
        })
        | Ok(PartValidation {
            guess: DoorPartResult::Computed { .. },
            validity: Skipped { .. },
        }) => panic!("Inconsistent PartValidation state"),
        Ok(PartValidation {
            validity: GuessSubmitted(IncorrectTooLow { guess }),
            ..
        }) => (format!("{guess} is too low"), '🔻'),
        Ok(PartValidation {
            validity: GuessSubmitted(IncorrectTooHigh { guess }),
            ..
        }) => (format!("{guess} is too high"), '🔺'),
        Ok(PartValidation {
            validity: GuessSubmitted(IncorrectTooManyGuesses { guess }),
            ..
        }) => (format!("{guess} is incorrect; too many guesses"), '❌'),
        Ok(PartValidation {
            guess: DoorPartResult::Computed { answer, .. },
            validity: GuessSubmitted(IncorrectOther),
        }) => (format!("{answer} is incorrect"), '❌'),
        Err(err) => (err.to_string(), '⛔'),
    };

    match part {
        Part::Part1 => f.write_fmt(format_args!(
            "Dec {day:2}, {year} - Part {part}: {message:54} {emoji}\n"
        )),
        Part::Part2 => f.write_fmt(format_args!(
            "               Part {part}: {message:54} {emoji}\n"
        )),
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (DoorDate { day, year }, progress) in &self.0 {
            match progress {
                Progress::Complete(result) => {
                    use Part::*;
                    write_answer(f, &result.date, Part1, &result.part1)?;
                    write_answer(f, &result.date, Part2, &result.part2)?;
                }
                _ => f.write_fmt(format_args!("Dec {day:2}, {year} - Part 1: {progress:?}\n               Part 2: {progress:?}\n"))?,
            }
        }

        Ok(())
    }
}

pub fn prefilled_screen() -> Result<impl std::io::Write + Send> {
    let mut screen = termion::screen::AlternateScreen::from(std::io::stdout());

    let (_, term_height) = termion::terminal_size()?;
    for _ in 0..term_height {
        write!(screen, "\n")?;
    }

    Ok(screen)
}

pub async fn process_progress_updates<S>(
    mut rx: mpsc::Receiver<DoorProgress>,
    mut screen: S,
    doors: &'static [DoorEntry],
) -> Table
where
    S: std::io::Write + Send,
{
    let mut table = Table(
        doors
            .iter()
            .map(|DoorEntry(date, ..)| (*date, Progress::Pending))
            .collect(),
    );

    while let Some(DoorProgress(date, progress)) = rx.recv().await {
        table.0.insert(date, progress);

        write!(screen, "{}{table}", termion::clear::All).unwrap();
        screen.flush().unwrap();
    }

    table
}
