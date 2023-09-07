use anyhow::{anyhow, bail, Context, Result}; // (xp) (thiserror in prod)
use aws_sdk_s3::{config, Client, Credentials, Region, ByteStream};
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::io::stdin;
use std::path::Path;
use sqlx::PgPool;
use tokio_stream::StreamExt;
use crate::db;

const ENV_CRED_KEY_ID: &str = "S3_KEY_ID";
const ENV_CRED_KEY_SECRET: &str = "S3_KEY_SECRET";
const REGION: &str = "REGION";
const BUCKET_NAME: &str = "BUCKET_NAME";
#[derive(Debug)]
pub struct S3 {
    pub client: Client,
}

pub struct AccountConfigQuery {
    pub key_id: String,
    pub key_secret: String,
    pub region: String,
    pub bucket_name: String,
}

impl S3 {
    pub async fn new(pool: &PgPool) -> Result<Self> {
        let client = Self::get_asw_client(pool).await?;
        Ok(Self { client })
    }
    pub async fn set_bucket_name(pool: &PgPool, bucket_name: &str) -> Result<()>{
        db::set_bucket_name(&pool, bucket_name).await?;
        Ok(())
    }
    pub async fn set_region(pool: &PgPool, region: &str) -> Result<()>{
        db::set_region(&pool, region).await?;
        Ok(())
    }
    pub async fn log_in(pool: &PgPool) -> Result<()>{
        if db::is_logged_in(&pool).await? {
            bail!("You are already logged in!");
        }

        println!("Paste your S3AccessKeyId: ");
        let mut key_id = String::new();
        stdin().read_line(&mut key_id)?;
        key_id = key_id.trim().to_string();

        println!("Paste your S3AccessKeySecret: ");
        let mut key_secret= String::new();
        stdin().read_line(&mut key_secret)?;
        key_secret = key_secret.trim().to_string();

        println!("Paste your region: ");
        let mut region= String::new();
        stdin().read_line(&mut region)?;
        region = region.trim().to_string();

        println!("Paste your bucket name: ");
        let mut bucket_name= String::new();
        stdin().read_line(&mut bucket_name)?;
        bucket_name = bucket_name.trim().to_string();

        let id = db::log_in(&pool, AccountConfigQuery{
            key_id,
            key_secret,
            region,
            bucket_name
        }).await?;
        Self::get_asw_client(pool).await?;
        println!("You are successfully logged in!");
        Ok(())
    }
    pub async fn log_out(pool: &PgPool) -> Result<()>{
        if !db::is_logged_in(&pool).await? {
            bail!("You are not logged in!");
        }
        db::log_out(&pool).await?;
        println!("You are successfully logged out!");
        Ok(())
    }
    pub async fn get_asw_client(pool: &PgPool) -> Result<Client> {
        let cfg = db::get_account_cfg(&pool).await?;
        let cred = Credentials::new(cfg.key_id, cfg.key_secret, None, None, "loaded-from-custom-.env");

        let region = Region::new(cfg.region.to_string());
        let conf_builder = config::Builder::new()
            .region(region)
            .credentials_provider(cred);
        let conf = conf_builder.build();

        let client = Client::from_conf(conf);

        Ok(client)
    }
    pub async fn upload_file(&self, pool: &PgPool, path: &Path) -> Result<()> {
        let bucket_name = db::get_bucket_name(&pool).await?;
        //Validate path
        if !path.exists() {
            bail!("Path {} does not exist", path.display());
        }

        let key = path.to_str().ok_or_else(|| anyhow!("Invalid path {path:?}"))?;

        if !path.is_file() {
            bail!("Path {} is not a file", path.display());
        }

        //Prepare data

        let body = ByteStream::from_path(&path).await?;
        let content_type = mime_guess::from_path(&path).first_or_octet_stream().to_string();

        let req = self
            .client
            .put_object()
            .bucket(&bucket_name)
            .key(key)
            .body(body)
            .content_type(content_type);

        req.send().await?;
        println!("Uploaded file {} to bucket {}", &key, &bucket_name);
        Ok(())
    }
    pub async fn delete_file(&self, pool: &PgPool, path: &Path) -> Result<()> {
        let bucket_name = db::get_bucket_name(&pool).await?;

        if !path.exists() {
            bail!("Path {} does not exist", path.display());
        }

        let key = path.to_str().ok_or_else(|| anyhow!("Invalid path {path:?}"))?;

        if !path.is_file() {
            bail!("Path {} is not a file", path.display());
        }

        self.client
            .delete_object()
            .bucket(&bucket_name)
            .key(key)
            .send()
            .await?;
        println!("Deleted file {} from bucket {}", &key, &bucket_name);
        Ok(())
    }
    pub async fn download_file(&self, pool: &PgPool, key: &Path, dir: &Path) -> Result<()> {
        let bucket_name = db::get_bucket_name(&pool).await?;
        //Validate key
        let key = key.to_str().ok_or_else(|| anyhow!("Invalid path {key:?}"))?;


        if !dir.is_dir() {
            bail!("Directory {} does not exist", dir.display())
        }

        let file_path = dir.join(key);
        let parent_dir = file_path.parent().ok_or_else(|| anyhow!("Invalid parent path for {:?}", file_path))?;
        if !parent_dir.exists() {
            create_dir_all(parent_dir)?;
        }

        let res = self
            .client
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
    pub async fn list_keys(&self, pool: &PgPool) -> Result<Vec<String>> {
        let bucket_name = db::get_bucket_name(&pool).await?;
        let req = self
            .client
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

}