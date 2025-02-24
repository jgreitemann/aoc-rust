use crate::api::*;
use crate::cli::*;
use crate::door::*;
use crate::output::*;
use crate::validation::*;

use anyhow::Result;
use anyhow::anyhow;
use std::any::Any;
use std::io::Write;
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::mpsc;

pub async fn aoc_main(doors: &'static [DoorEntry]) -> Result<()> {
    let opts = Options::parse();

    let filter_day = opts.day;
    let doors = doors
        .iter()
        .filter(move |entry| filter_day.is_none_or(|filter_day| entry.0.day == filter_day));

    let client = caching_client(opts.empty_cache)?;

    let (tx, rx) = mpsc::channel(25);
    let updater_task = tokio::task::spawn(process_progress_updates(
        rx,
        prefilled_screen()?,
        doors.clone(),
    ));

    let result = tokio::task::LocalSet::new()
        .run_until(run_door_tasks(tx, doors, client, &opts))
        .await;

    let final_table = updater_task.await?;
    if result.is_ok() {
        print!("{final_table}");
    }
    std::io::stdout().lock().flush()?;

    result
}

async fn handle_door(
    DoorEntry(date, door_fn): &'static DoorEntry,
    client: Arc<impl AoCClient>,
    progress_sender: mpsc::Sender<DoorProgress>,
    opts: Options,
) -> Result<ValidationResult> {
    progress_sender
        .send(DoorProgress(*date, Progress::DownloadingInput))
        .await?;
    let input = client.get_input(date).await?;
    progress_sender
        .send(DoorProgress(*date, Progress::DownloadingDay))
        .await?;
    let status = client.get_day(date).await?;
    progress_sender
        .send(DoorProgress(*date, Progress::ComputingAnswer))
        .await?;
    let parts_considered_solved = if opts.skip_solved {
        status.solved_count()
    } else {
        0
    };
    let (answer_tx, answer_rx) = tokio::sync::oneshot::channel();
    rayon::spawn(move || {
        let result =
            std::panic::catch_unwind(|| door_fn(input.trim_end(), parts_considered_solved))
                .unwrap_or_else(|e| DoorResult {
                    part1: Err(panic_as_anyhow_error(e.as_ref())),
                    part2: Err(panic_as_anyhow_error(e.as_ref())),
                });
        answer_tx.send(result).unwrap()
    });
    let answer = answer_rx.await?;
    progress_sender
        .send(DoorProgress(*date, Progress::ValidatingAnswer))
        .await?;
    validate_answer(
        date,
        answer,
        &status,
        opts.validation_mode(),
        client.as_ref(),
    )
    .await
}

async fn run_door_tasks(
    tx: mpsc::Sender<DoorProgress>,
    doors: impl IntoIterator<Item = &'static DoorEntry>,
    client: impl AoCClient + 'static,
    opts: &Options,
) -> Result<()> {
    let client = Arc::new(client);
    let mut set = tokio::task::JoinSet::new();
    for entry in doors {
        set.spawn_local(handle_door(entry, client.clone(), tx.clone(), opts.clone()));
    }

    while let Some(result) = set.join_next().await {
        let result = result??;
        tx.send(DoorProgress(result.date, Progress::Complete(result)))
            .await?;
    }
    Ok(())
}

fn panic_as_anyhow_error(panic_error: &dyn Any) -> anyhow::Error {
    if let Some(panic_message) = panic_error
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic_error.downcast_ref::<&str>().map(Deref::deref))
    {
        anyhow!("Panic: {panic_message}")
    } else {
        anyhow!("Panic: cause unknown")
    }
}
