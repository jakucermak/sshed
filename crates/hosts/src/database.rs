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

pub async fn define_schema<C: Connection>(db: &Surreal<C>) -> surrealdb::Result<()> {
    //Define Tag
    db.query("DEFINE TABLE tag SCHEMAFULL;").await?;
    db.query("DEFINE FIELD name ON TABLE tag TYPE string;")
        .await?;
    //Define Group
    db.query("DEFINE TABLE group SCHEMAFULL;").await?;
    db.query("DEFINE FIELD name ON TABLE group TYPE string;")
        .await?;

    db.query(
        "DEFINE TABLE tagged TYPE RELATION IN tag OUT host SCHEMAFULL PERMISSIONS NONE;

    -- ------------------------------
    -- FIELDS
    -- ------------------------------

    DEFINE FIELD in ON tagged TYPE record<tag> PERMISSIONS FULL;
    DEFINE FIELD out ON tagged TYPE record<host> PERMISSIONS FULL;",
    )
    .await?;

    db.query(
        "DEFINE TABLE groupped TYPE RELATION IN group OUT host SCHEMAFULL PERMISSIONS NONE;

    -- ------------------------------
    -- FIELDS
    -- ------------------------------

    DEFINE FIELD in ON groupped TYPE record<group> PERMISSIONS FULL;
    DEFINE FIELD out ON groupped TYPE record<host> PERMISSIONS FULL;",
    )
    .await?;

    Ok(())
}

pub async fn set_namespace<C: Connection>(db: &Surreal<C>) -> Result<(), surrealdb::Error> {
    db.use_ns("hosts").use_db("hosts").await
}
