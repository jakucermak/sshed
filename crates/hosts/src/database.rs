use surrealdb::{
    engine::{
        local::{Db, RocksDb},
        remote::ws::{Client, Ws},
    },
    opt::auth::Root,
    Connection, Surreal,
};

pub async fn set_db(cfg_storage: &str) -> Result<Surreal<Db>, surrealdb::Error> {
    Surreal::new::<RocksDb>(cfg_storage).await
}

pub async fn set_remote_db(addr: &str) -> Result<Surreal<Client>, surrealdb::Error> {
    Surreal::new::<Ws>(addr).await
}

pub async fn login<C: Connection>(
    db: &Surreal<C>,
    user: &str,
    pwd: &str,
) -> Result<surrealdb::opt::auth::Jwt, surrealdb::Error> {
    db.signin(Root {
        username: user,
        password: pwd,
    })
    .await
}

pub async fn set_namespace<C: Connection>(db: &Surreal<C>) -> Result<(), surrealdb::Error> {
    db.use_ns("hosts").use_db("hosts").await
}
