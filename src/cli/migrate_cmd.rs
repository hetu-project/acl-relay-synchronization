use crate::common::config;
use crate::db;
use clap::{ArgGroup, Parser};

#[derive(Debug, Clone, Parser)]
#[command(group(ArgGroup::new("exclusive").args(&["db_url", "config_file"])))]
pub struct MigrateCmd {
    #[arg(short, long)]
    db_url: Option<String>,

    #[arg(short, long)]
    config_file: Option<String>,
}

impl MigrateCmd {
    /// Handles the execution of the configuration subcommand.  
    pub async fn run(&self) {
        if let Some(db_url) = &self.db_url {
            if let Ok(url) = url::Url::parse(db_url) {
                let db_name = url.path().trim_start_matches('/');
                let base_url = url.as_str().trim_end_matches(db_name);
                db::setup_db(base_url, db_name).await.unwrap();
            }
        }

        if let Some(config) = &self.config_file {
            let config = config::Config::load_config(config.into()).unwrap();

            if let Ok(url) = url::Url::parse(&config.database.db_url) {
                let db_name = url.path().trim_start_matches('/');
                let base_url = url.as_str().trim_end_matches(db_name);
                db::setup_db(base_url, db_name).await.unwrap();
            }
        }
    }
}
