use aws_config::{
    BehaviorVersion,
    SdkConfig,
};
use color_eyre::eyre::{
    Report,
    Result,
};
use tokio::sync::OnceCell;

static ONCE: OnceCell<SdkConfig> = OnceCell::const_new();

pub async fn aws_config() -> Result<&'static SdkConfig> {
    Ok(ONCE
        .get_or_try_init(|| async {
            let prod = std::env::var("PROD");
            let is_prod = matches!(prod, Ok(s) if s == "true" || s == "1" || s == "yes");
            let config = if is_prod {
                let config = aws_config::defaults(BehaviorVersion::latest())
                    .region("us-east-1")
                    .load()
                    .await;
                todo!()
            } else {
                aws_config::load_from_env().await
            };
            Ok::<_, Report>(config)
        })
        .await?)
}
