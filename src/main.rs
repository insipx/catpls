mod bot;
mod rest;
mod content;

use color_eyre::eyre::{eyre, Result};
use tokio::task::JoinHandle;

async fn flatten<T>(handle: JoinHandle<Result<T>>) -> Result<T> {
    match handle.await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(err)) => Err(err),
        Err(_) => Err(eyre!("handling failed")),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().pretty().init();
    tokio::try_join!(
        flatten(tokio::task::spawn(rest::web_server())),
        flatten(tokio::task::spawn(bot::catbot()))
    )?;
    Ok(())
}
