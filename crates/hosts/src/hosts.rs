use std::{
    fs::File,
    io::{BufReader, Error, ErrorKind, Read, Result},
    path::PathBuf,
};

pub mod database;
pub mod host;
use host::{
    table::{Group, Tag},
    EnhancedHost,
};
use ssh2_config::{ParseRule, SshConfig};
use surrealdb::{sql::Thing, Connection, Surreal};

#[derive(Debug)]
pub struct Hosts {}

impl Hosts {
    pub async fn parse_config<C: Connection>(db: &Surreal<C>, path: PathBuf) -> Result<()> {
        let mut reader = BufReader::new(File::open(path)?);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        let blocks: Vec<&str> = content
            .split("\n\n")
            .map(|block| block.trim())
            .filter(|block| !block.is_empty())
            .collect();

        match exctract_host(blocks, db).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
    }

    pub async fn get_all_hosts<C: Connection>(db: &Surreal<C>) -> Result<Vec<EnhancedHost>> {
        let hosts: Vec<EnhancedHost> = db.select("host").await.unwrap();

        Ok(hosts)
    }
}

async fn exctract_host<C: Connection>(blocks: Vec<&str>, db: &Surreal<C>) {
async fn exctract_host<C: Connection>(blocks: Vec<&str>, db: &Surreal<C>) -> Result<()> {
    for block in blocks {
        let mut lines: Vec<&str> = block.lines().collect();
        let mut groups: Vec<Thing> = vec![];
        let mut tags: Vec<Thing> = vec![];
        let mut comment = None;

        extract_metadata(&mut lines, &mut groups, &mut tags, &mut comment, db).await;

        let host_config = lines.join("\n");
        let mut host_reader = host_config.as_bytes();

        if let Ok(config) = SshConfig::default().parse(&mut host_reader, ParseRule::STRICT) {
            if let Some(host) = config.get_hosts().get(1).cloned() {
                let enh_host = EnhancedHost {
                    host: host.into(),
                    comment,
                };
                EnhancedHost::create_or_update(db, enh_host).await.unwrap();
            }
        }
    }
}

async fn extract_metadata<C: Connection>(
    lines: &mut Vec<&str>,
    groups: &mut Vec<Thing>,
    tags: &mut Vec<Thing>,
    comment: &mut Option<String>,
    db: &Surreal<C>,
) {
    while let Some(line) = lines.first() {
        if line.starts_with("#--(") {
            // Parse groups only if present
            if let Some(group_str) = line.strip_prefix("#--(").and_then(|s| s.strip_suffix(")")) {
                let group_names: Vec<String> =
                    group_str.split(',').map(|s| s.trim().to_string()).collect();

                for group_name in group_names {
                    if let Ok(group_id) = Group::create_or_update(group_name, db).await {
                        groups.push(group_id);
                    }
                }
            }
            lines.remove(0);
        } else if line.starts_with("#--[") {
            // Parse tags only if present
            if let Some(tag_str) = line.strip_prefix("#--[").and_then(|s| s.strip_suffix("]")) {
                let tag_names: Vec<String> =
                    tag_str.split(',').map(|s| s.trim().to_string()).collect();

                for tag_name in tag_names {
                    if let Ok(tag_id) = Tag::create_or_update(tag_name, db).await {
                        tags.push(tag_id);
                    }
                }
            }
            lines.remove(0);
        } else if line.starts_with("# ") {
            *comment = Some(line[2..].to_string());
            lines.remove(0);
        } else {
            break;
        }
    }
}

#[cfg(test)]
mod tests {}
