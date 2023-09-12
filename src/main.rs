#[allow(unused)]

mod s3;
mod db;
mod error;

use anyhow::{bail, Result};
use std::env;
use std::path::Path;
use dotenv::dotenv;
use crate::s3::{S3};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let pool = db::init().await.or_else(|err| {
        bail!("Failed to connect to database: {}", err);
    })?;

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        bail!("Missing command");
    }
    match args[1].as_str() {
        "help" => {
            println!("Commands:");
            println!("  log-in");
            println!("  log-out");
            println!("  set-bucket <bucket-name>");
            println!("  set-region <region>");
            println!("  list");
            println!("  delete <file-name>");
            println!("  upload <file-name> <file-path>");
            println!("  download <file-name> <file-path>");
            println!("  content-type <file-name>");
            println!("  file-size <file-name>");
            return Ok(());
        },
        "log-in" => {
            S3::log_in(&pool).await?;
            return Ok(());
        },
        "log-out" => {
            S3::log_out(&pool).await?;
            return Ok(());
        },
        "set-bucket" => {
        },
        "set-region" => {
        },
        "list" => {
        },
        "delete" => {
        },
        "upload" => {
        },
        "download" => {
        },
        "content-type" => {
        },
        "file-size" => {
        },
        _ => {}
    }

    let s3 = S3::new(&pool).await?;

    match args[1].as_str() {
        "set-bucket" => {
            S3::set_bucket_name(&pool, &args[2]).await?;
        },
        "set-region" => {
            S3::set_region(&pool, &args[2]).await?;
        },
        "list" => {
            let keys = s3.list_keys(&pool).await?;
            for key in &keys {
                let size = s3.file_size(&pool, Path::new(&key)).await?;
                let content_type = s3.content_type(&pool, Path::new(&key)).await?;
                println!("{} (File size: {}, Content type: {})", key, size, content_type);
            }


        },
        "delete" => {
            s3.delete_file(&pool,Path::new(&args[2])).await?;
        },
        "upload" => {
            s3.upload_file( &pool,Path::new(&args[2]), Path::new(&args[3])).await?;
        },
        "download" => {
            s3.download_file( &pool,Path::new(&args[2]), Path::new(&args[3])).await?;
        },
        "content-type" => {
            let content_type = s3.content_type( &pool,Path::new(&args[2])).await?;
            println!("File size: {} Bytes", content_type);
        },
        "file-size" => {
            let file_size = s3.file_size( &pool,Path::new(&args[2])).await?;
            println!("{}", file_size);
        },
        _ => {
            bail!("Unknown command {}", args[1]);
        }
    }

    Ok(())
}

