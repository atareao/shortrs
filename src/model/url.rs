use serde::{Serialize, Deserialize};
use sqlx::{sqlite::{SqlitePool, SqliteRow}, query, Row};
use chrono::{DateTime, Utc};
use tracing::{debug, info};

use super::radix::to_d36;


#[derive(Debug, Serialize, Deserialize, Clone)]
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
            id: row.get("url_id"),
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
        let sql = "INSERT OR IGNORE INTO urls (src, num, active, created_at, updated_at)
                   VALUES($1, $2, $3, $4, $4) RETURNING *";
        debug!("Query: {}", sql);
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
    pub async fn read_from_url(pool: &SqlitePool, src: &str) -> Result<Self, sqlx::Error>{
        let sql = "SELECT * FROM urls WHERE src = $1 LIMIT 1";
        debug!("Query: {}", sql);
        debug!("src: {}", src);
        query(sql)
            .bind(src)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }

    pub async fn update_or_create(pool: &SqlitePool, src: &str) -> Result<Self, sqlx::Error>{
        match Self::read_from_url(pool, src).await{
            Ok(url) => {
                let url = Self::increase(pool, &url).await.unwrap();
                println!("Exists: {:?}", &url);
                Ok(url)
            },
            Err(e) => {
                info!("Read or create: {}", e);
                Self::create(pool, src).await
            }
        }
    }

    pub async fn exists(pool: &SqlitePool, url_id: i64) -> bool{
        let sql = "SELECT count(*) FROM urls WHERE url_id = $1";
        debug!("Query: {}", sql);
        match query(sql)
            .bind(url_id)
            .map(|row: SqliteRow| -> i64 {row.get(0)})
            .fetch_one(pool)
            .await {
                Ok(value) => value > 0,
                Err(e) => {
                    info!("Error on exists {}", e);
                    false
                }
            }
    }

    pub async fn read(pool: &SqlitePool, url_id: i64) -> Result<Self, sqlx::Error>{
        let sql = "SELECT * FROM urls WHERE url_id = $1";
        debug!("Query: {}", sql);
        query(sql)
            .bind(url_id)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }

    pub async fn increase(pool: &SqlitePool, url: &Self) -> Result<Self, sqlx::Error>{
        let sql = "UPDATE urls SET num = $2, active = $3,
                   created_at = $4, updated_at = $5 FROM urls
                   WHERE url_id = $1
                   RETURNING *";
        debug!("Url to increase: {:?}", url);
        debug!("Query: {}", sql);
        query(sql)
            .bind(url.id)
            .bind(url.num + 1)
            .bind(url.active)
            .bind(url.created_at)
            .bind(Utc::now())
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }
    pub async fn update(pool: &SqlitePool, url: Self) -> Result<Self, sqlx::Error>{
        let sql = "UPDATE urls SET num = $2, active = $3,
                   created_at = $4, updated_at = $5 FROM urls
                   WHERE url_id = $1
                   RETURNING *";
        debug!("Query: {}", sql);
        query(sql)
            .bind(url.id)
            .bind(url.num)
            .bind(url.active)
            .bind(url.created_at)
            .bind(Utc::now())
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }

    pub async fn delete(pool: &SqlitePool, url_id: i64) -> Result<Self, sqlx::Error>{
        let sql = "DELETE from urls WHERE url_id = $1
                   RETURNING *";
        debug!("Query: {}", sql);
        query(sql)
            .bind(url_id)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }
}

#[cfg(test)]
mod url_test {
    use sqlx::{
        self,
        Pool,
        sqlite::{
            Sqlite,
            SqlitePoolOptions,
        },
        migrate::{
            Migrator,
            MigrateDatabase
        }
    };
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    use std::{env, path::Path, str::FromStr};
    use tracing::{info, debug};
    use super::Url;

    const DB_URL: &'static str = "sqlite:test.db";

    async fn setup() -> Pool<Sqlite>{
        tracing_subscriber::registry()
            .with(EnvFilter::from_str("debug").unwrap())
            .with(tracing_subscriber::fmt::layer())
            .init();
        teardown().await;
        if !sqlx::Sqlite::database_exists(DB_URL).await.unwrap(){
            sqlx::Sqlite::create_database(DB_URL).await.unwrap();
        }
        let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let migrations = Path::new(&crate_dir).join("./migrations");
        info!("{}", &migrations.display());
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(DB_URL)
            .await
            .expect("Pool failed");

        Migrator::new(migrations)
            .await
            .unwrap()
            .run(&pool)
            .await
        .unwrap();
        pool
    }

    #[allow(unused_must_use)]
    async fn teardown() {
        tokio::fs::remove_file("test.db").await;
        tokio::fs::remove_file("test.db-shm").await;
        tokio::fs::remove_file("test.db-wal").await;
    }

    #[tokio::test]
    async fn test_read_url(){
        // Start and prepare
        let pool = setup().await;
        // Test
        let src = "https://atareao.es";
        let url = Url::create(&pool, src).await.unwrap();
        debug!("Url created: {:?}", url);
        match Url::update_or_create(&pool, src).await{
            Ok(url) => println!("Url: {:?}", url),
            Err(e) => {
                println!("Porque no encuentra nada");
                println!("Error: {}", e);
            },

        };
        // End and Clean
        teardown().await;
    }
}

