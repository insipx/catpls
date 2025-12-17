use std::path::PathBuf;
use color_eyre::eyre::WrapErr;
use color_eyre::eyre::{
    Result,
    bail,
};
use directories::BaseDirs;
use crate::bot::identity::CatBotIdentity;
use xmtp_mls::identity::IdentityStrategy;
mod identity;

pub async fn catbot() -> Result<()> {
    let env = std::env::var("NODE_URL").wrap_err("catbot: NODE_URL unset")?;
    let chain_id = std::env::var("CHAIN_ID").wrap_err("catbot: CHAIN_ID unset")?.parse()?;
    let aws_key_id = std::env::var("AWS_KEY_ID").wrap_err("catbot: AWS_KEY_ID unset")?.parse()?;
     let nonce = std::env::var("XMTP_IDENTITY_NONCE")
            .wrap_err("unable to parse xmtp identity nonce from env")?.parse()?;
    let identity = CatBotIdentity::new(chain_id, aws_key_id, nonce).await?;
    let builder = Client::builder(identity.strategy(false));
    // generate a wallet and display on screen
    todo!()
}

fn sqlite_path() -> Result<PathBuf> {
    if let Some(base) = BaseDirs::new() {
        let path = base.data_dir().join("catbot").join("local_db");
        Ok(path.into())
    } else {
        bail!("Could not detect XDG base directories")
    }
}
