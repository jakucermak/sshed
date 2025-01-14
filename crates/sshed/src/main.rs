use std::{
    io::Error,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};

use cli::parse_args;
use config::{read_config, AppConfig};
use hosts::{database, Hosts};
use notify::{
    event::{DataChange, ModifyKind},
    Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use surrealdb::Surreal;

fn monitor_cfg_change(path: &PathBuf, appconfig: Arc<Mutex<AppConfig>>) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        tx,
        Config::default().with_poll_interval(Duration::from_secs(5)),
    )?;

    watcher.watch(&path, RecursiveMode::NonRecursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                if event.kind == EventKind::Modify(ModifyKind::Data(DataChange::Content)) {
                    if let Ok(mut config) = appconfig.lock() {
                        *config = read_config(path);
                    }
                }
            }
            Err(e) => println!("Watch error: {:?}", e),
        }
    }
    Ok(())
}

async fn parse_ssh_config<C: surrealdb::Connection>(
    db: &Surreal<C>,
    configuration: Arc<Mutex<AppConfig>>,
) -> Result<(), Error> {
    let config = configuration.lock().unwrap();
    let path = match config.general.as_ref() {
        Some(g) => match g.ssh_config_path.as_ref() {
            Some(p) => PathBuf::from_str(p).expect("Invalid path string"),
            None => panic!("SSH config path not found"),
        },
        None => panic!("SSH config path not found"),
    };

    match Hosts::parse_config(&db, path).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    let args = parse_args();
    let cfg = match args {
        Ok(ref pth) => Arc::new(Mutex::new(read_config(pth))),
        Err(e) => {
            eprint!("{}", e);
            std::process::exit(1);
        }
    };

    let config_clone = Arc::clone(&cfg);
    std::thread::spawn(move || {
        if let Err(e) = monitor_cfg_change(&args.unwrap(), config_clone) {
            eprintln!("Error monitoring file: {}", e);
        }
    });

    // let storage_path = match &cfg.lock().unwrap().general {
    //     Some(p) => p.storage.as_ref().unwrap(),
    //     None => panic!("No storage path provided"),
    // };

    // let db = database::set_db("./db").await.unwrap();
    let db = database::set_remote_db("127.0.0.1:8000").await.unwrap();
    database::login(&db, "root", "root").await.unwrap();
    database::set_namespace(&db).await.unwrap();
    database::define_schema(&db).await.unwrap();

    parse_ssh_config(&db, cfg)
        .await
        .expect("Failed to parse SSH config");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
