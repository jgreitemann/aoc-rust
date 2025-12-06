use crate::api::AnswerResponse;
use crate::door::*;
use crate::validation::*;

use anyhow::Result;
use termion::screen::IntoAlternateScreen;
use tokio::sync::mpsc;

use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::num::NonZero;

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
        }) => (
            format!("{answer} ({time:.0$})", significant_decimals(time, 3)),
            '⭐',
        ),
        Ok(PartValidation {
            guess: DoorPartResult::Computed { answer, .. },
            validity: DisparateAnswer { correct },
        }) => (format!("{answer}, but correct answer was {correct}"), '💢'),
        Ok(PartValidation {
            guess: DoorPartResult::Computed { answer, time },
            validity: GuessSubmitted(Correct),
        }) => (
            format!("{answer} ({time:.0$})", significant_decimals(time, 3)),
            '🌟',
        ),
        Ok(PartValidation {
            guess: DoorPartResult::Computed { answer, .. },
            validity: GuessSubmitted(GuessedTooRecently),
        }) => (
            format!("{answer} (unable to submit; guessed too recently)"),
            '🕑',
        ),
        Ok(PartValidation {
            guess: DoorPartResult::Computed { answer, time },
            validity: Unknown,
        }) => (
            format!("{answer} ({time:.0$})", significant_decimals(time, 3)),
            '🤷',
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
            guess: DoorPartResult::Computed { answer, .. },
            validity: GuessSubmitted(IncorrectTooLow),
            ..
        }) => (format!("{answer} is too low"), '🔻'),
        Ok(PartValidation {
            guess: DoorPartResult::Computed { answer, .. },
            validity: GuessSubmitted(IncorrectTooHigh),
            ..
        }) => (format!("{answer} is too high"), '🔺'),
        Ok(PartValidation {
            guess: DoorPartResult::Computed { answer, .. },
            validity: GuessSubmitted(IncorrectTooManyGuesses),
            ..
        }) => (format!("{answer} is incorrect; too many guesses"), '❌'),
        Ok(PartValidation {
            guess: DoorPartResult::Computed { answer, .. },
            validity: GuessSubmitted(IncorrectOther),
        }) => (format!("{answer} is incorrect"), '❌'),
        Err(err) => (err.to_string(), '⛔'),
    };

    match part {
        Part::Part1 => writeln!(f, "Dec {day:2}, {year} - Part {part}: {message:54} {emoji}"),
        Part::Part2 => writeln!(f, "               Part {part}: {message:54} {emoji}"),
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            writeln!(f, "No solutions implemented for days matching filter")?;
        }
        for (DoorDate { day, year }, progress) in &self.0 {
            match progress {
                Progress::Complete(result) => {
                    use Part::*;
                    write_answer(f, &result.date, Part1, &result.part1)?;
                    write_answer(f, &result.date, Part2, &result.part2)?;
                }
                _ => writeln!(
                    f,
                    "Dec {day:2}, {year} - Part 1: {progress:?}\n               Part 2: {progress:?}"
                )?,
            }
        }

        Ok(())
    }
}

pub fn prefilled_screen() -> Result<impl std::io::Write + Send> {
    let mut screen = std::io::stdout().into_alternate_screen()?;

    let (_, term_height) = termion::terminal_size()?;
    for _ in 0..term_height {
        writeln!(screen)?;
    }

    Ok(screen)
}

pub async fn process_progress_updates<S>(
    mut rx: mpsc::Receiver<DoorProgress>,
    mut screen: S,
    doors: impl IntoIterator<Item = &'static DoorEntry>,
) -> Table
where
    S: std::io::Write + Send,
{
    let mut table = Table(
        doors
            .into_iter()
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

fn significant_decimals(duration: &time::Duration, significant_places: usize) -> usize {
    let seconds = duration.as_seconds_f64();
    let Some(non_zero_unit) =
        NonZero::new((seconds / time::convert::Second::per_t::<f64>(time::convert::Day)) as i64)
            .or_else(|| {
                NonZero::new(
                    (seconds / time::convert::Second::per_t::<f64>(time::convert::Hour)) as i64,
                )
            })
            .or_else(|| {
                NonZero::new(
                    (seconds / time::convert::Second::per_t::<f64>(time::convert::Minute)) as i64,
                )
            })
            .or_else(|| NonZero::new((seconds) as i64))
            .or_else(|| {
                NonZero::new(
                    (seconds * time::convert::Millisecond::per_t::<f64>(time::convert::Second))
                        as i64,
                )
            })
            .or_else(|| {
                NonZero::new(
                    (seconds * time::convert::Microsecond::per_t::<f64>(time::convert::Second))
                        as i64,
                )
            })
            .or_else(|| {
                NonZero::new(
                    (seconds * time::convert::Nanosecond::per_t::<f64>(time::convert::Second))
                        as i64,
                )
            })
    else {
        return 0;
    };

    let leading_digits = non_zero_unit.get().checked_ilog10().unwrap_or(0) as usize + 1;

    significant_places.saturating_sub(leading_digits)
}
