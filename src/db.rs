use crate::error::DBError;
use crate::s3;
use s3::AccountConfigQuery;
use serde::{Deserialize, Serialize};
use sqlx;
use sqlx::postgres::PgPool;
use std::env;
#[derive(Serialize, Deserialize)]
pub struct Event<'a> {
    pub key_id: &'a str,
    pub event_type: &'a str,
    pub bucket_name: &'a str,
    pub file_name: &'a str,
    pub file_type: &'a str,
    pub file_size: &'a str,
}
pub async fn init() -> Result<PgPool, DBError> {
    let db_url = env::var("DATABASE_URL").unwrap();
    let pool = PgPool::connect(&db_url).await.unwrap();
    Ok(pool)
}

pub async fn is_logged_in(pool: &PgPool) -> Result<bool, DBError> {
    let rows_number = sqlx::query!(
        r"
        SELECT COUNT(*) as count
        FROM s3_account;"
    )
    .fetch_one(pool)
    .await?;
    Ok(rows_number.count.unwrap() > 0)
}
pub async fn log_in(pool: &PgPool, cfg: AccountConfigQuery) -> Result<i32, DBError> {
    let sql_response = sqlx::query!(
        r#"
        INSERT INTO s3_account (id, key_id, key_secret, region, bucket_name) VALUES ($1, $2, $3, $4, $5)
        RETURNING id;"#,
        1,
        cfg.key_id,
        cfg.key_secret,
        cfg.region,
        cfg.bucket_name
    )
        .fetch_one(pool)
        .await?;

    Ok(sql_response.id)
}

pub async fn log_out(pool: &PgPool) -> Result<i32, DBError> {
    let sql_response = sqlx::query!(
        r#"
        DELETE FROM s3_account WHERE id = 1
        RETURNING id;"#,
    )
    .fetch_one(pool)
    .await?;

    Ok(sql_response.id)
}

pub async fn set_bucket_name(pool: &PgPool, bucket_name: &str) -> Result<i32, DBError> {
    let sql_response = sqlx::query!(
        r#"
        UPDATE s3_account
        SET bucket_name = $1
        WHERE id = 1
        RETURNING id;"#,
        bucket_name
    )
    .fetch_one(pool)
    .await?;

    Ok(sql_response.id)
}

pub async fn set_region(pool: &PgPool, region: &str) -> Result<i32, DBError> {
    let sql_response = sqlx::query!(
        r#"
        UPDATE s3_account
        SET region = $1
        WHERE id = 1
        RETURNING id;"#,
        region
    )
    .fetch_one(pool)
    .await?;

    Ok(sql_response.id)
}

pub async fn get_account_cfg(pool: &PgPool) -> Result<AccountConfigQuery, DBError> {
    let account_number = sqlx::query!(
        r"
        SELECT COUNT(*) as count
        FROM s3_account
        WHERE id = 1;"
    )
    .fetch_one(pool)
    .await?;
    if account_number.count.unwrap() == 0 {
        return Err(DBError::new("You are not logged in!".to_string()));
    }
    let sql_response = sqlx::query!(
        r#"
        SELECT key_id, key_secret, region, bucket_name
        FROM s3_account
        WHERE id = 1;"#,
    )
    .fetch_one(pool)
    .await?;
    Ok(AccountConfigQuery {
        key_id: sql_response.key_id,
        key_secret: sql_response.key_secret,
        region: sql_response.region,
        bucket_name: sql_response.bucket_name,
    })
}

pub async fn get_bucket_name(pool: &PgPool) -> Result<String, DBError> {
    let sql_response = sqlx::query!(
        r#"
        SELECT bucket_name
        FROM s3_account
        WHERE id = 1;"#,
    )
    .fetch_one(pool)
    .await?;
    Ok(sql_response.bucket_name)
}

pub async fn add_event<'a>(pool: &PgPool, event: Event<'a>) -> Result<i32, DBError> {
    let sql_responce = sqlx::query!(
      r#"
      INSERT INTO s3_events (key_id, event_type, bucket_name, file_name, file_type, file_size) VALUES ($1, $2, $3, $4, $5, $6)
      RETURNING id;"#,
        event.key_id,
        event.event_type,
        event.bucket_name,
        event.file_name,
        event.file_type,
        event.file_size
    )
        .fetch_one(pool)
        .await?;
    Ok(sql_responce.id)
}
