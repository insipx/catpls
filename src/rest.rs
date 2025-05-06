use color_eyre::eyre::{self, Result};
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;
use warp::Filter;
use warp::reply::WithHeader;

#[derive(Serialize, Deserialize)]
pub struct CatId {
    id: String,
}

pub async fn web_server() -> Result<()> {
    let store = CatStore::load_from_env().await?;
    let cat = cat("image/jpeg", &store);
    Ok(())
}

pub struct CatStore {
    bucket_endpoint: String,
    bucket_name: String,
    client: aws_sdk_s3::Client,
}

impl CatStore {
    async fn load_from_env() -> Result<Self> {
        let bucket_endpoint = std::env::var("CATPLS_BUCKET_ENDPOINT")?;
        let bucket_name = std::env::var("CATPLS_BUCKET_NAME")?;
        let config = aws_config::load_from_env().await;
        let mut config = config.into_builder();
        config.set_endpoint_url(Some(bucket_endpoint.clone()));
        let config = config.build();
        let client = aws_sdk_s3::Client::new(&config);
        Ok(Self {
            bucket_endpoint,
            bucket_name,
            client,
        })
    }

    async fn get_cat(&self, id: &str) -> Result<Vec<u8>> {
        let cat = self
            .client
            .get_object()
            .set_bucket(Some(self.bucket_name.clone()))
            .key(id)
            .send()
            .await?;
        Ok(cat.body.collect().await?.into_bytes().to_vec())
    }
}

fn cat(
    content_type: &str,
    store: &CatStore,
) -> impl Filter<Extract = (WithHeader<Vec<u8>>,), Error = warp::Rejection> + Copy {
    warp::get()
        .and(warp::path("cat"))
        .and(warp::path::param::<String>())
        .and_then(|id: String| async move {
            let cat = store.get_cat(&id).await
            .inspect_err(|e| tracing::error!("{}", e))
            .map_err(|_| warp::reject::custom(
                    r#"
                    Cats encountered an error processing your request
                    　　　　　／＞　　フ
                    　　　　　| 　_　 _ l
                    　 　　　／` ミ＿xノ
                    　　 　 /　　　 　 |
                    　　　 /　 ヽ　　 ﾉ
                    　 　 │　　| | |
                    "#))?;
            // upload to S3 with a key
            // reply with a UUID
            // let id = Uuid::new_v4();
            Ok(warp::reply::with_header(vec![], "Content-Type", content_type.to_owned()))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cat() {
        let filter = cat("image/jpeg");

        let res = warp::test::request()
            .path("/cat/12345")
            .reply(&filter)
            .await;
        println!("{:?}", res);
        // assert_eq!(res.status(), 405, "GET is not allowed");
    }
}
