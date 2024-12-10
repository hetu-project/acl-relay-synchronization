use super::entities::prelude::{
    LastUpdateActiveModel, LastUpdateEntity, NostrEventActiveModel, NostrEventColumn,
    NostrEventEntity,
};
use super::migration::Migrator;
use crate::common::config::DatabaseConfig;
use crate::common::error;
use chrono;
use sea_orm::*;
use sea_orm_migration::prelude::*;
use std::{sync::Arc, time::Duration};

pub async fn setup_db(req_url: &str, db_name: &str) -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(req_url).await?;
    let db = match db.get_database_backend() {
        DbBackend::MySql => {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE IF NOT EXISTS `{}`;", db_name),
            ))
            .await?;

            let url = format!("{}/{}", req_url, db_name);
            Database::connect(&url).await?
        }
        DbBackend::Postgres => {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("DROP DATABASE IF EXISTS \"{}\";", db_name),
            ))
            .await?;
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE \"{}\";", db_name),
            ))
            .await?;

            let url = format!("{}/{}", req_url, db_name);
            Database::connect(&url).await?
        }
        DbBackend::Sqlite => db,
    };

    let schema_manager = SchemaManager::new(&db);
    Migrator::up(&db.clone(), None).await?;

    Ok(db)
}

#[derive(Debug, Default, Clone)]
pub struct Storage {
    pub conn: Arc<DatabaseConnection>,
}

impl Storage {
    pub async fn new(config: DatabaseConfig) -> Self {
        //let url = format!("{}/{}", config.url, config.db_name);
        let mut opt = ConnectOptions::new(&config.db_url);
        opt.max_connections(config.max_connect_pool)
            .min_connections(config.min_connect_pool)
            .connect_timeout(Duration::from_secs(config.connect_timeout))
            .acquire_timeout(Duration::from_secs(config.acquire_timeout));

        let db = Database::connect(opt.clone())
            .await
            .expect("failed to connect to database");

        Self { conn: Arc::new(db) }
    }

    pub async fn get_last_update(&self, init: u64) -> error::Result<u64> {
        match LastUpdateEntity::find().one(self.conn.as_ref()).await? {
            Some(last) => Ok(last.last_update as u64),
            None => {
                let new_last_update = LastUpdateActiveModel {
                    last_update: Set(init as i64),
                    updated_at: Set(chrono::Utc::now().into()),
                    ..Default::default()
                };
                new_last_update.insert(self.conn.as_ref()).await?;
                Ok(init)
            }
        }
    }

    pub async fn update_last_update(&self, last: u64) -> error::Result<()> {
        if let Some(mut last_update) = LastUpdateEntity::find()
            .one(self.conn.as_ref())
            .await?
            .map(|l| l.into_active_model())
        {
            last_update.last_update = Set(last as i64);
            last_update.updated_at = Set(chrono::Utc::now().into());

            last_update.update(self.conn.as_ref()).await?;
        }

        Ok(())
    }

    pub async fn is_event_existed(&self, id: String) -> Option<()> {
        if NostrEventEntity::find()
            .filter(NostrEventColumn::EventId.eq(id))
            .one(self.conn.as_ref())
            .await
            .is_ok()
        {
            Some(())
        } else {
            None
        }
    }

    pub async fn add_new_event(&self, id: String) -> error::Result<()> {
        let new_event_id = NostrEventActiveModel {
            event_id: Set(id),
            updated_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        };

        new_event_id.insert(self.conn.as_ref()).await?;

        Ok(())
    }
}
