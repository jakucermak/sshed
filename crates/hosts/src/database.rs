use surrealdb::{
    engine::local::{Db, RocksDb},
    Surreal,
};

pub async fn set_db(cfg_storage: &str) -> Result<Surreal<Db>, surrealdb::Error> {
    Surreal::new::<RocksDb>(cfg_storage).await
}

pub async fn set_namespace(db: &Surreal<Db>) -> Result<(), surrealdb::Error> {
    db.use_ns("hosts").use_db("hosts").await
}
