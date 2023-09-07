use crate::error::DBError;
use serde::{Deserialize, Serialize};
use sqlx;
use sqlx::postgres::PgPool;
use std::env;
use s3::AccountConfigQuery;
use crate::s3;

pub type PoolConn = sqlx::pool::PoolConnection<sqlx::Postgres>;

#[derive(Serialize, Deserialize)]
pub struct Note {
    pub id: i32,
    pub name: String,
    pub text: String,
}

pub async fn connection(pool: &PgPool) -> Result<PoolConn, DBError> {
    PgPool::acquire(pool)
        .await
        .map_err(|e| DBError::new(500, format!("Failed getting database connection: {}", e)))
}

pub async fn init() -> Result<PgPool, DBError> {
    let db_url = env::var("DATABASE_URL").expect("Database url must be set!");
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
        return Err(DBError::new(409, "You are not logged in".to_string()));
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

// pub async fn filter(
//     conn: &mut PoolConn,
//     page: usize,
//     size: usize,
//     q: String,
// ) -> Result<Vec<Note>, DBError> {
//     let mut query_str = "%".to_owned();
//     query_str.push_str(q.as_str());
//     query_str.push_str("%");
//
//     let offset: i64 = (page * size) as i64;
//     let limit: i64 = size as i64;
//
//     let sql_response = sqlx::query!(
//         r#"
//             SELECT id, name, text
//             FROM notes
//             WHERE name LIKE $1
//             ORDER BY id
//             OFFSET $2
//             LIMIT $3;
//         "#,
//         &query_str,
//         offset,
//         limit
//     )
//         .fetch_all(conn)
//         .await?;
//
//     let mut result = Vec::new();
//
//     for i in 0..sql_response.len() {
//         result.push(Note {
//             id: sql_response.get(i).unwrap().id,
//             name: sql_response.get(i).unwrap().name.clone(),
//             text: sql_response.get(i).unwrap().text.clone(),
//         });
//     }
//
//     Ok(result)
// }
//
// pub async fn delete(conn: &mut PoolConn, id: i32) -> Result<i32, DBError> {
//     let sql_response = sqlx::query!(
//         r#"
//         DELETE FROM notes
//         WHERE id = $1
//         RETURNING id;"#,
//         id
//     )
//         .fetch_one(conn)
//         .await?;
//     Ok(sql_response.id)
// }

// pub async fn update(
//     conn: &mut PoolConn,
//     id: i32,
//     note: NoteQuery,
// ) -> Result<i32, DBError> {
//     let sql_response = sqlx::query!(
//         r#"
//         UPDATE notes
//         SET (name, text) = ($1, $2)
//         WHERE id = $3
//         RETURNING id;"#,
//         note.name,
//         note.text,
//         id
//     )
//         .fetch_one(conn)
//         .await?;
// 
//     Ok(sql_response.id)
// }

// pub async fn find(conn: &mut PoolConn, id: i32) -> Result<Note, DBError> {
//     let sql_response = sqlx::query!(
//         r#"
//         SELECT name, text
//         FROM notes
//         WHERE id = $1;"#,
//         id
//     )
//         .fetch_one(conn)
//         .await?;
//     Ok(Note {
//         id,
//         name: sql_response.name,
//         text: sql_response.text,
//     })
// }