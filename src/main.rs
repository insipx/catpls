mod bot;
mod config;
mod content;
mod rest;
use color_eyre::eyre::{
    Result,
    eyre,
};
pub(crate) use config::aws_config;
use tokio::task::JoinHandle;

#[macro_use]
extern crate tracing;

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
        // flatten(tokio::task::spawn(bot::catbot()))
    )?;
    Ok(())
}
