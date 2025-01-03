use std::{
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::Surreal;

use cli::parse_args;
use config::{read_config, AppConfig};
use hosts::host::EnhancedHost;
use hosts::Hosts;
use notify::{
    event::{DataChange, ModifyKind},
    Config, EventKind, RecommendedWatcher, RecursiveMode, Result, Watcher,
};

fn monitor_cfg_change(path: &PathBuf, appconfig: Arc<Mutex<AppConfig>>) -> Result<()> {
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

async fn parse_ssh_config(
    configuration: Arc<Mutex<AppConfig>>,
    db: &Surreal<Db>,
) -> Vec<EnhancedHost> {
    let config = configuration.lock().unwrap();
    let path = match config.general.as_ref() {
        Some(g) => match g.ssh_config_path.as_ref() {
            Some(p) => PathBuf::from_str(p).expect("Invalid path string"),
            None => panic!("SSH config path not found"),
        },
        None => panic!("SSH config path not found"),
    };

    match Hosts::parse_config(path, db).await {
        Ok(r) => r,
        Err(_) => todo!(),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = parse_args();
    let configuration = match args {
        Ok(ref pth) => Arc::new(Mutex::new(read_config(pth))),
        Err(e) => {
            eprint!("{}", e);
            std::process::exit(1);
        }
    };

    let config_clone = Arc::clone(&configuration);
    std::thread::spawn(move || {
        if let Err(e) = monitor_cfg_change(&args.unwrap(), config_clone) {
            eprintln!("Error monitoring file: {}", e);
        }
    });
    let cfg_storage = Arc::clone(&configuration)
        .lock()
        .unwrap()
        .general
        .as_ref()
        .unwrap()
        .storage
        .as_ref()
        .unwrap()
        .clone();
    let db = Surreal::new::<RocksDb>(cfg_storage)
        .await
        .map_err(|e| notify::Error::from(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    parse_ssh_config(configuration, &db).await;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
