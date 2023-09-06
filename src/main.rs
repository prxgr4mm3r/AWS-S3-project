#[allow(unused)]

use anyhow::{anyhow, bail, Context, Result}; // (xp) (thiserror in prod)
use aws_sdk_s3::{config, Client, Credentials, Region, ByteStream};
use std::env;
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use tokio_stream::StreamExt;
use dotenv::dotenv;
const ENV_CRED_KEY_ID: &str = "S3_KEY_ID";
const ENV_CRED_KEY_SECRET: &str = "S3_KEY_SECRET";
const BUCKET_NAME: &str = "aws-s3-rust";
const REGION: &str = "eu-north-1";


#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let client= get_asw_client(REGION)?;

    delete_file(&client, BUCKET_NAME, Path::new("src/main.rs")).await?;

    let keys = list_keys(&client, BUCKET_NAME).await?;
    println!("{}", keys.join("\n"));
    Ok(())
}
async fn delete_file(client: &Client, bucket_name: &str, path: &Path) -> Result<()> {
    if !path.exists() {
        bail!("Path {} does not exist", path.display());
    }

    let key = path.to_str().ok_or_else(|| anyhow!("Invalid path {path:?}"))?;

    client
        .delete_object()
        .bucket(bucket_name)
        .key(key)
        .send()
        .await?;

    Ok(())
}
async fn download_file(client: &Client, bucket_name: &str, key: &Path, dir: &Path) -> Result<()> {
    //Validate key
    let key = key.to_str().ok_or_else(|| anyhow!("Invalid path {key:?}"))?;

    if !dir.is_dir() {
        bail!("Directory {} does not exist", dir.display())
    }

    let file_path = dir.join(key);
    println!("file_path: {}", file_path.display());
    let parent_dir = file_path.parent().ok_or_else(|| anyhow!("Invalid parent path for {:?}", file_path))?;
    if !parent_dir.exists() {
        create_dir_all(parent_dir)?;
    }
    println!("file_path: {}", parent_dir.display());

    let res = client
        .get_object()
        .bucket(bucket_name)
        .key(key)
        .send().await?;

    let mut data = res.body;
    let file = File::create(&file_path)?;
    let mut buf_writer = BufWriter::new(&file);
    while let Some(bytes) = data.try_next().await? {
        buf_writer.write(&bytes)?;
    }
    buf_writer.flush()?;

    Ok(())
}
async fn upload_file(client: &Client, bucket_name: &str, path: &Path) -> Result<()> {
    //Validate path
    if !path.exists() {
        bail!("Path {} does not exist", path.display());
    }

    let key = path.to_str().ok_or_else(|| anyhow!("Invalid path {path:?}"))?;

    //Prepare data

    let body = ByteStream::from_path(&path).await?;
    let content_type = mime_guess::from_path(&path).first_or_octet_stream().to_string();

    let req = client
        .put_object()
        .bucket(bucket_name)
        .key(key)
        .body(body)
        .content_type(content_type);

    req.send().await?;

    Ok(())
}
async fn list_keys(client: &Client, bucket_name: &str) -> Result<Vec<String>> {
    let req = client
        .list_objects_v2()
        .prefix("")
        .bucket(bucket_name);

    let res = req.send().await?;

    let keys = res.contents.unwrap_or_default();

    Ok(
        keys
            .iter()
            .filter_map(|o| o.key.as_ref())
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    )
}
fn get_asw_client(region: &str) -> Result<Client> {
    let key_id = env::var(ENV_CRED_KEY_ID).context("Missing S3_KEY_ID")?;
    let key_secret = env::var(ENV_CRED_KEY_SECRET).context("Missing S3_KEY_SECRET")?;

    let cred = Credentials::new(key_id, key_secret, None, None, "loaded-from-custom-.env");

    let region = Region::new(region.to_string());
    let conf_builder = config::Builder::new()
        .region(region)
        .credentials_provider(cred);
    let conf = conf_builder.build();

    let client = Client::from_conf(conf);

    Ok(client)
}
