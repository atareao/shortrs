use serde::{Serialize, Deserialize};
use serde_yaml::Error;
use std::process;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Configuration{
    url: String,
    log_level: String,
    db_url: String,
    port: u16,
    username: String,
    password: String,
    per_page: i64,
}

impl Configuration {
    pub fn new(content: &str) -> Result<Configuration, Error>{
        serde_yaml::from_str(content)
    }
    pub fn get_url(&self) -> &str{
        &self.url
    }
    pub fn get_log_level(&self) -> &str{
        &self.log_level
    }
    pub fn get_db_url(&self) -> &str{
        &self.db_url
    }
    pub fn get_port(&self) -> u16{
        self.port
    }
    pub fn get_username(&self) -> &str{
        &self.username
    }
    pub fn get_password(&self) -> &str{
        &self.password
    }
    pub fn get_page(&self) -> i64{
        self.per_page
    }

    pub async fn read() -> Self{
        let content = match tokio::fs::read_to_string("config.yml")
            .await {
                Ok(value) => value,
                Err(e) => {
                    println!("Error with config file `config.yml`: {}",
                        e.to_string());
                    process::exit(0);
                }
            };
        match Configuration::new(&content){
            Ok(configuration) => configuration,
            Err(e) => {
                println!("Error with config file `config.yml`: {}",
                    e.to_string());
                process::exit(0);
            }
        }
    }
}
