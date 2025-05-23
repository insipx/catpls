#![allow(unused)]
use color_eyre::eyre::{self, Result};
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;
use warp::{Filter, Rejection};
use warp::reply::WithHeader;
use futures::future::TryFuture;
use std::sync::Arc;
use std::convert::Infallible;

#[derive(Serialize, Deserialize)]
pub struct CatId {
    id: String,
}

pub async fn web_server() -> Result<()> {
    let store = Arc::new(CatStore::load_from_env().await?);
    let cat = cat(store.clone());
    warp::serve(cat).run(([127, 0, 0, 1], 3000)).await;
    Ok(())
}

fn cat(store: Arc<CatStore>) -> impl Filter<Extract = (WithHeader<Vec<u8>>,), Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("cat"))
        .and(warp::path::param::<String>())
        .and(with_store(store))
        .and_then(|id: String, store: Arc<CatStore>| async {
            fetch_cat(id, store).await
        })
}

fn with_store(store: Arc<CatStore>) -> impl Filter<Extract = (Arc<CatStore>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || store.clone())
}

async fn fetch_cat(id: String, store: Arc<CatStore>) -> Result<WithHeader<Vec<u8>>, Rejection> {
    let cat = store.get_cat(&id).await
        .inspect_err(|e| tracing::error!("{}", e))
        .map_err(|e| CatRejected { reason: e.to_string() })?;
    let content_type = "image/jpeg";
    Ok(warp::reply::with_header(vec![0, 1, 2, 3], "Content-Type", content_type.to_owned()))
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

struct CatRejected {
    reason: String
}

impl std::fmt::Debug for CatRejected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cat = include_str!("../sad_ascii.cat");
        writeln!(f,"\n{}", cat)?;
        write!(f, "shared braincell unable to complete request: {}\n", self.reason)
    }
}

impl warp::reject::Reject for CatRejected {}

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
