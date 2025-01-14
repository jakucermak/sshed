
use config::Storage;
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

#[derive(Debug)]
enum AddressType {
    WebAddress,
    FilePath,
    Invalid,
}

fn is_valid_ip(ip: &str) -> bool {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return false;
    }

    parts.iter().all(|part| {
        if let Ok(_num) = part.parse::<u8>() {
            true
        } else {
            false
        }
    })
}

fn is_valid_domain(domain: &str) -> bool {
    if domain.is_empty() || domain.starts_with('.') || domain.ends_with('.') {
        return false;
    }

    let parts: Vec<&str> = domain.split('.').collect();
    if parts.len() < 2 {
        return false;
    }

    parts
        .iter()
        .all(|part| !part.is_empty() && part.chars().all(|c| c.is_alphanumeric() || c == '-'))
}

fn is_valid_port(port: &str) -> bool {
    if let Ok(_num) = port.parse::<u16>() {
        true
    } else {
        false
    }
}

fn is_valid_path_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-')
}

fn is_valid_file_path(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }

    // Handle special cases
    if path == "." || path == ".." {
        return true;
    }

    let parts: Vec<&str> = path.split('/').collect();

    // For absolute paths (starting with /)
    if path.starts_with('/') {
        return parts[1..].iter().all(|part| {
            part.is_empty() || // Allow consecutive slashes
            *part == "." ||
            *part == ".." ||
            is_valid_path_segment(part)
        });
    }

    // For relative paths
    parts.iter().all(|part| {
        part.is_empty() || // Allow consecutive slashes
        *part == "." ||
        *part == ".." ||
        is_valid_path_segment(part)
    })
}

fn identify_address(input: &str) -> AddressType {
    // Check if it's a web address (IP:port or domain:port)
    if let Some((host, port)) = input.rsplit_once(':') {
        if (is_valid_ip(host) || is_valid_domain(host)) && is_valid_port(port) {
            return AddressType::WebAddress;
        }
    }
    // Check if it's a file path
    else if is_valid_file_path(input) {
        return AddressType::FilePath;
    }

    AddressType::Invalid
}

pub async fn create_connection(storage: &Storage) -> surrealdb::Result<Surreal<Client>> {
    let path = storage.path.as_ref().unwrap();

    match identify_address(path) {
        AddressType::WebAddress => set_remote_db(path).await,
        AddressType::FilePath => todo!(),
        AddressType::Invalid => todo!(),
    }
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
