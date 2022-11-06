use crate::api::*;
use crate::cli::*;
use crate::door::*;
use crate::output::*;
use crate::validation::*;

use anyhow::Result;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::mpsc;

pub async fn aoc_main(doors: &'static [DoorEntry]) -> Result<()> {
    let opts = Options::parse();
    let client = caching_client(opts.empty_cache)?;

    let (tx, rx) = mpsc::channel(25);
    let updater_task = tokio::task::spawn(process_progress_updates(rx, prefilled_screen()?, doors));

    let result = run_door_tasks(tx, doors, client).await;

    let final_table = updater_task.await?;
    if result.is_ok() {
        print!("{final_table}");
    }
    std::io::stdout().lock().flush()?;

    result
}

async fn handle_door<C>(
    DoorEntry(date, door_fn): &DoorEntry,
    client: Arc<C>,
    progress_sender: mpsc::Sender<DoorProgress>,
) -> Result<ValidationResult>
where
    C: AoCClient + Send + Sync,
{
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
    let answer = door_fn(&input);
    progress_sender
        .send(DoorProgress(*date, Progress::ValidatingAnswer))
        .await?;
    validate_answer(date, answer, &status, client.as_ref()).await
}

async fn run_door_tasks<C>(
    tx: mpsc::Sender<DoorProgress>,
    doors: &'static [DoorEntry],
    client: C,
) -> Result<()>
where
    C: AoCClient + Send + Sync + 'static,
{
    let client = Arc::new(client);
    let mut set = tokio::task::JoinSet::new();
    for entry in doors {
        set.spawn(handle_door(entry, client.clone(), tx.clone()));
    }

    while let Some(result) = set.join_next().await {
        let result = result??;
        tx.send(DoorProgress(result.date, Progress::Complete(result)))
            .await?;
    }
    Ok(())
}
