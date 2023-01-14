use serde::{Serialize, Deserialize};
use sqlx::{sqlite::{SqlitePool, SqliteRow}, query, Row};
use chrono::{DateTime, Utc};

use super::radix::to_d36;


#[derive(Debug, Serialize, Deserialize)]
pub struct Url{
    id: i64,
    src: String,
    num: u32,
    active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Url{
    pub fn get_src(&self) -> &str{
        &self.src
    }
    pub fn get_id(&self) -> i64{
        self.id
    }
    pub fn get_num(&self) -> u32{
        self.num
    }
    pub fn new(src: String) -> Self{
        let active = true;
        let created_at = Utc::now();
        let updated_at = created_at.clone();
        Self{
            id: 0,
            src,
            num: 0,
            active,
            created_at,
            updated_at,
        }
    }
    pub fn get_url(self) -> String{
        let value: u32 = self.id.try_into().unwrap();
        to_d36(value)
    }

    fn from_row(row: SqliteRow) -> Self{
        Self{
            id: row.get("id"),
            src: row.get("src"),
            num: row.get("num"),
            active: row.get("active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    pub async fn create(pool: &SqlitePool, src: &str) -> Result<Self, sqlx::Error>{
        let num = 0;
        let active = true;
        let created_at = Utc::now();
        let updated_at = created_at.clone();
        let sql = "INSERT INTO urls (src, num, active, created_at, updated_at)
                   VALUES($1, $2, $3, $4, $4) RETURNIN *";
        query(sql)
            .bind(src)
            .bind(num)
            .bind(active)
            .bind(created_at)
            .bind(updated_at)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }

    pub async fn exists(pool: &SqlitePool, id: i64) -> bool{
        let sql = "SELECT count(*) FROM urls WHERE id = $1";
        match query(sql)
            .bind(id)
            .map(|row: SqliteRow| -> i64 {row.get(0)})
            .fetch_one(pool)
            .await {
                Ok(value) => value > 0,
                Err(e) => {
                    tracing::info!("Error on exists {}", e);
                    false
                }
            }
    }

    pub async fn read(pool: &SqlitePool, id: i64) -> Result<Self, sqlx::Error>{
        let sql = "SELECT * FROM urls WHERE id = $1";
        query(sql)
            .bind(id)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }

    pub async fn update(pool: &SqlitePool, url: Self) -> Result<Self, sqlx::Error>{
        let sql = "UPDATE urls SET num = $2, active = $3,
                   created_at = $4, updated_at = $5 FROM urls
                   WHERE id = $1
                   RETURNING *";
        query(sql)
            .bind(url.id)
            .bind(url.num)
            .bind(url.active)
            .bind(url.created_at)
            .bind(url.updated_at)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }

    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<Self, sqlx::Error>{
        let sql = "DELETE from urls WHERE id = $1
                   RETURNING *";
        query(sql)
            .bind(id)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }

}
