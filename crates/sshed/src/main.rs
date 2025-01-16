use std::{
    path::PathBuf,
    sync::{self, Arc, Mutex},
    time::Duration,
};

use db::DbRuntime;
use ui::HelloWorld;

use cli::parse_args;
use config::{read_config, AppConfig};
use gpui::{App, AppContext, VisualContext, WindowOptions};
use notify::{
    event::{DataChange, ModifyKind},
    Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use ssh_parser::{self, SshParser};

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

fn main() {
    env_logger::init();

    let args = parse_args();

    let app = App::new();

    let cfg = match args {
        Ok(ref pth) => Arc::new(Mutex::new(read_config(pth))),
        Err(e) => {
            eprint!("{}", e);
            std::process::exit(1);
        }
    };

    app.run(move |cx: &mut AppContext| {
        let config_clone = Arc::clone(&cfg);
        let args_clone = args.unwrap();
        std::thread::spawn(move || {
            if let Err(e) = monitor_cfg_change(&args_clone, config_clone) {
                eprintln!("Error monitoring file: {}", e);
            }
        });

        let db = DbRuntime::new();
        SshParser::init(db, cfg.clone()).unwrap();

        cx.open_window(WindowOptions::default(), |cx| {
            cx.new_view(|_cx| HelloWorld {
                text: "World".into(),
            })
        })
        .unwrap();
    });
}
