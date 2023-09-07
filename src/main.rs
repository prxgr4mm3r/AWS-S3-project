#[allow(unused)]

mod s3;
mod db;
mod error;

use anyhow::{bail, Context, Result}; // (xp) (thiserror in prod)
use std::env;
use std::io::{Write};
use std::path::Path;
use tokio_stream::StreamExt;
use dotenv::dotenv;
use crate::s3::{S3};

const BUCKET_NAME: &str = "aws-s3-rust";
const REGION: &str = "eu-north-1";


#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let pool = db::init().await.unwrap();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        bail!("Missing command");
    }
    match args[1].as_str() {
        "login" => {
            S3::log_in(&pool).await?;
            return Ok(());
        },
        "logout" => {
            S3::log_out(&pool).await?;
            return Ok(());
        },
        _ => {}
    }

    let mut s3 = S3::new(&pool).await?;

    // db::log_in(&pool, AccountConfigQuery {
    //     key_id: "AKIAT65KOOZV5K6W6DEQ".to_string(),
    //     key_secret: "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".to_string(),
    //     region: REGION.to_string(),
    //     bucket_name: BUCKET_NAME.to_string(),
    // }).await?;
    // db::log_out(&pool).await?;

    match args[1].as_str() {
        "set-bucket" => {
            S3::set_bucket_name(&pool, &args[2]).await?;
        },
        "set-region" => {
            S3::set_region(&pool, &args[2]).await?;
        },
        "list" => {
            let keys = s3.list_keys(&pool).await?;
            println!("Keys: {:?}", keys);
        },
        "delete" => {
            s3.delete_file(&pool,Path::new(&args[2])).await?;
        },
        "upload" => {
            s3.upload_file( &pool,Path::new(&args[2])).await?;
        },
        "download" => {
            s3.download_file( &pool,Path::new(&args[2]), Path::new(&args[3])).await?;
        },
        _ => {
            bail!("Unknown command {}", args[1]);
        }
    }

    Ok(())
}

