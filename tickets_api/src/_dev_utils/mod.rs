use tokio::sync::OnceCell;
use tracing::info;

use crate::model::ModelManager;

mod dev_db;

pub async fn init_dev() {
    static INIT: OnceCell<()> = OnceCell::const_new();

    INIT.get_or_init(|| async {
        info!("{:<12} - init_dev", "FOR_DEV_ONLY");
        dev_db::init_dev_deb().await.unwrap();
    })
    .await;
}

pub async fn init_test_db() -> ModelManager {
    static INIT: OnceCell<ModelManager> = OnceCell::const_new();

    let manager = INIT
        .get_or_init(|| async {
            init_dev().await;
            ModelManager::new().await.unwrap()
        })
        .await;

    manager.clone()
}
