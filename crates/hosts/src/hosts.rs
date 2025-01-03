use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

pub mod host;
use host::{group::Group, tag::Tag, EnhancedHost};
use ssh2_config::{ParseRule, SshConfig};
use surrealdb::{engine::local::Db, sql::Thing, Surreal};

#[derive(Debug)]
pub struct Hosts {
    pub hosts: Vec<EnhancedHost>,
}

impl Hosts {
    pub async fn parse_config(
        path: PathBuf,
        db: &Surreal<Db>,
    ) -> std::io::Result<Vec<EnhancedHost>> {
        let mut reader = BufReader::new(File::open(path)?);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        let blocks: Vec<&str> = content
            .split("\n\n")
            .map(|block| block.trim())
            .filter(|block| !block.is_empty())
            .collect();

        let mut enhanced_hosts = Vec::new();

        exctract_host(blocks, &mut enhanced_hosts, db).await;
        Ok(enhanced_hosts)
    }
}

async fn exctract_host(
    blocks: Vec<&str>,
    enhanced_hosts: &mut Vec<EnhancedHost>,
    db: &Surreal<Db>,
) {
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
                enhanced_hosts.push(EnhancedHost {
                    host,
                    comment,
                    groups,
                    tags,
                });
            }
        }
    }
}

async fn extract_metadata(
    lines: &mut Vec<&str>,
    groups: &mut Vec<Thing>,
    tags: &mut Vec<Thing>,
    comment: &mut Option<String>,
    db: &Surreal<Db>,
) {
    while let Some(line) = lines.first() {
        if line.starts_with("#--(") {
            // Parse groups only if present
            if let Some(group_str) = line.strip_prefix("#--(").and_then(|s| s.strip_suffix(")")) {
                let group_names: Vec<String> =
                    group_str.split(',').map(|s| s.trim().to_string()).collect();

                for group_name in group_names {
                    if let Ok(group_id) = Group::create_or_update(group_name, vec![], db).await {
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
                    if let Ok(tag_id) = Tag::create_or_update(tag_name, vec![], db).await {
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
