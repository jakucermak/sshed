use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use cli::parse_args;
use config::{read_config, AppConfig};
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

fn main() {
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

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
