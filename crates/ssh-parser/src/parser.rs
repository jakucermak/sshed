use std::{
    io::Error,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex, MutexGuard},
};

use config::AppConfig;
use hosts::Hosts;
use surrealdb::Surreal;

pub async fn parse_ssh_config<C: surrealdb::Connection>(
    db: &Surreal<C>,
    configuration: Arc<Mutex<AppConfig>>,
) -> Result<(), Error> {
    let config = configuration.lock().unwrap();
    let path = get_path(config);

    if path.to_str().unwrap().ends_with('*') {
        let paths = expand_path(path);
        for path in paths {
            Hosts::parse_config(&db, path).await?;
        }
        Ok(())
    } else {
        Hosts::parse_config(&db, path).await
    }
}

fn expand_path(path: PathBuf) -> Vec<PathBuf> {
    let parent = path.parent().expect("Path must have a parent directory");
    let pattern = path
        .file_name()
        .expect("Path must have a filename")
        .to_str()
        .expect("Path must be valid UTF-8");
    let mut paths = Vec::new();

    process_directory(parent, pattern, &mut paths);
    paths
}

fn process_directory(dir: &std::path::Path, pattern: &str, paths: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        // !("Entries: {:?}", entries);
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    process_directory(&path, pattern, paths);
                } else {
                    paths.push(entry.path());
                }
            }
        }
    }
}

fn get_path(config: MutexGuard<'_, AppConfig>) -> PathBuf {
    let path = match config.general.as_ref() {
        Some(g) => match g.ssh_config_path.as_ref() {
            Some(p) => PathBuf::from_str(p).expect("Invalid path string"),
            None => panic!("SSH config path not found"),
        },
        None => panic!("SSH config path not found"),
    };
    path
}
