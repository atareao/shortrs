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

#[derive(Serialize, Deserialize)]
pub struct ShortUrl{
    src: String,
    short: String,
    num: u32,
}

impl Url{
    pub fn get_src(&self) -> &str{
        &self.src
    }
    pub fn get_short(&self) -> ShortUrl{
        ShortUrl {
            src: self.src.to_string(),
            short: self.get_url(),
            num: self.num,
        }
    }
    pub fn get_num(&self) -> u32{
        self.num
    }

    pub fn get_url(&self) -> String{
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
        info!("Url create");
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
        info!("Url aread_from_url");
        let sql = "SELECT * FROM urls WHERE src = $1 LIMIT 1";
        debug!("Query: {}", sql);
        query(sql)
            .bind(src)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }

    pub async fn read_or_create(pool: &SqlitePool, src: &str) -> Result<Self, sqlx::Error>{
        info!("Url read_or_create");
        match Self::read_from_url(pool, src).await{
            Ok(url) => {
                Ok(url)
            },
            Err(e) => {
                Self::create(pool, src).await
            }
        }
    }

    pub async fn exists(pool: &SqlitePool, id: i64) -> bool{
        info!("Url exists");
        let sql = "SELECT count(*) FROM urls WHERE id = $1";
        debug!("Query: {}", sql);
        match query(sql)
            .bind(id)
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

    pub async fn read(pool: &SqlitePool, id: i64) -> Result<Self, sqlx::Error>{
        info!("Url read");
        let sql = "SELECT * FROM urls WHERE id = $1";
        debug!("Query: {}", sql);
        query(sql)
            .bind(id)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }

    pub async fn read_all(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error>{
        info!("Url read");
        let sql = "SELECT * FROM urls";
        debug!("Query: {}", sql);
        query(sql)
            .map(Self::from_row)
            .fetch_all(pool)
            .await
    }

    pub async fn increase(pool: &SqlitePool, url: &Self) -> Result<Self, sqlx::Error>{
        info!("Url increase");
        let sql = "UPDATE urls SET num = $2, active = $3,
                   created_at = $4, updated_at = $5
                   WHERE id = $1 RETURNING *";
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
        info!("update");
        let sql = "UPDATE urls SET num = $2, active = $3,
                   created_at = $4, updated_at = $5 FROM urls
                   WHERE id = $1
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

    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<Self, sqlx::Error>{
        info!("Url delete");
        let sql = "DELETE from urls WHERE id = $1
                   RETURNING *";
        debug!("Query: {}", sql);
        query(sql)
            .bind(id)
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
    use std::{env, path::Path};
    use super::Url;

    async fn setup(db: &str) -> Pool<Sqlite>{
        let db_url = format!("sqlite:{}", db);
        teardown(db).await;
        if !sqlx::Sqlite::database_exists(&db_url).await.unwrap(){
            sqlx::Sqlite::create_database(&db_url).await.unwrap();
        }
        let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let migrations = Path::new(&crate_dir).join("./migrations");
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&db_url)
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
    async fn teardown(db: &str) {
        tokio::fs::remove_file(db).await;
        tokio::fs::remove_file(format!("{}-shm", db)).await;
        tokio::fs::remove_file(format!("{}-wal", db)).await;
    }

    #[tokio::test]
    async fn test_create(){
        let db = "test-create.db";
        // Start and prepare
        let pool = setup(db).await;
        // Test
        let src = "https://google.es";
        let url = Url::create(&pool, src).await.unwrap();
        assert!(url.get_src() == src);
        assert!(url.get_num() == 0);
        // End and Clean
        teardown(db).await;
    }

    #[tokio::test]
    async fn test_increase(){
        let db = "test-increase.db";
        // Start and prepare
        let pool = setup(db).await;
        // Test
        let src = "https://atareao.es";
        let url = Url::create(&pool, src).await.unwrap();
        let new_url = Url::increase(&pool, &url).await.unwrap();
        assert!(new_url.get_src() == src);
        assert!(new_url.get_num() == 1);
        // End and Clean
        teardown(db).await;
    }

    #[tokio::test]
    async fn test_read_from_url(){
        let db = "test-read-from-url.db";
        // Start and prepare
        let pool = setup(db).await;
        // Test
        let src = "https://atareao.es";
        let url = Url::create(&pool, src).await.unwrap();
        let read_url = Url::read_from_url(&pool, src).await.unwrap();
        assert!(read_url.get_src() == url.get_src());
        assert!(read_url.get_num() == url.get_num());
        // End and Clean
        teardown(db).await;
    }

    #[tokio::test]
    async fn test_delete(){
        let db = "test-delete.db";
        // Start and prepare
        let pool = setup(db).await;
        // Test
        let src = "https://atareao.es";
        let url = Url::create(&pool, src).await.unwrap();
        let _result = Url::delete(&pool, url.id).await;
        assert!(Url::exists(&pool, url.id).await == false);
        // End and Clean
        teardown(db).await;
    }
}

