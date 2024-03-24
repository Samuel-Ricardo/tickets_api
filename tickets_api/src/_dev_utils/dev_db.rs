use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{fs, path::PathBuf, time::Duration};
use tracing::info;

use crate::{
    ctx::CTX,
    model::{user::User, ModelManager},
    service::user::UserService,
};

type DB = Pool<Postgres>;

//  NOTE: Hardcided to prevent deployed system db update
const PG_DEV_POSTGRES_URL: &str = "postgres://postgres:welcome@db:5432/postgres";
const PG_DEV_APP_URL: &str = "postgres://app_user:dev_only_pwd@db_test:5432/app_db";

/* -- sql files -- */

const SQL_RECREATE_DB: &str = "sql/dev_initial/00-recreate-db.sql";
const SQL_DIR: &str = "sql/dev_initial";

const DEMO_PWD: &str = "welcome";

pub async fn init_dev_deb() -> Result<(), Box<dyn std::error::Error>> {
    info!("{:<12} - init_dev_deb", "FOR_DEV_ONLY");

    // NOTE: Create the app_db/app_user with postgres user
    {
        let root_db = new_db_pool(PG_DEV_POSTGRES_URL).await?;
        pexec(&root_db, SQL_RECREATE_DB).await?;
    }

    // INFO: Get SQL Files
    let mut paths: Vec<PathBuf> = fs::read_dir(SQL_DIR)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();
    paths.sort();

    let app_db = new_db_pool(PG_DEV_APP_URL).await?;
    for path in paths {
        if let Some(path) = path.to_str() {
            let path = path.replace('\\', "/"); // INFO: For Windows

            if path.ends_with(".sql") && path != SQL_RECREATE_DB {
                pexec(&app_db, &path).await?;
            }
        }
    }

    let manager = ModelManager::new().await.unwrap();
    let ctx = CTX::root_ctx();

    let demo1_user: User = UserService::first_by_username(&ctx, &manager, "demo1")
        .await
        .unwrap()
        .unwrap();

    UserService::update_pwd(&ctx, &manager, demo1_user.id, DEMO_PWD).await?;
    info!(
        "{:<12} - init_dev_deb: done [set_demo1_user]",
        "FOR_DEV_ONLY"
    );

    Ok(())
}

pub async fn pexec(db: &DB, file: &str) -> Result<(), sqlx::Error> {
    info!("{:<12} - pexec: {file}", "FOR_DEV_ONLY");

    let content = fs::read_to_string(file)?;

    // FIXME: Make the split more sql proof
    let sqls: Vec<&str> = content.split(';').collect();

    for sql in sqls {
        sqlx::query(sql).execute(db).await?;
    }

    Ok(())
}

pub async fn new_db_pool(db_con_url: &str) -> Result<DB, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_millis(500))
        .connect(db_con_url)
        .await
}
