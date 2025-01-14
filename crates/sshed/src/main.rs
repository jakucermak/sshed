use std::{
    io::Error,
    path::PathBuf,
    sync::{self, Arc, Mutex},
    time::Duration,
};

use db as database;

use cli::parse_args;
use config::{read_config, AppConfig};
use notify::{
    event::{DataChange, ModifyKind},
    Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use ssh_parser;

fn monitor_cfg_change(path: &PathBuf, appconfig: Arc<Mutex<AppConfig>>) -> notify::Result<()> {
    let (tx, rx) = sync::mpsc::channel();
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

    ssh_parser::parse_ssh_config(&db, cfg)
        .await
        .expect("Failed to parse SSH config");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
