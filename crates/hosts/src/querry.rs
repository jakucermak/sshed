use crate::host::{
    table::{Group, Tag},
    HostRecord,
};
use std::collections::HashSet;
use surrealdb::{Connection, Error, Surreal};

#[derive(Debug)]
pub struct SearchResults {
    pub hosts: Vec<HostRecord>,
    pub tags: Vec<Tag>,
    pub groups: Vec<Group>,
}

#[derive(Debug)]
pub struct HostSearch {
    selected_tags: HashSet<String>,
    selected_groups: HashSet<String>,
}

impl HostSearch {
    pub fn new() -> Self {
        Self {
            selected_tags: HashSet::new(),
            selected_groups: HashSet::new(),
        }
    }

    /// Search for suggestions based on partial input
    pub async fn suggest<C: Connection>(
        db: &Surreal<C>,
        pattern: &str,
    ) -> Result<SearchResults, Error> {
        let pattern = pattern.to_lowercase();

        // Find matching hosts
        let hosts: Vec<HostRecord> = db
            .query("SELECT * FROM host WHERE string::lowercase(host.name) CONTAINS $pattern")
            .bind(("pattern", pattern.clone()))
            .await?
            .take(0)?;

        // Find matching tags
        let tags: Vec<Tag> = db
            .query("SELECT name FROM tag WHERE string::lowercase(name) CONTAINS $pattern")
            .bind(("pattern", pattern.clone()))
            .await?
            .take(0)?;

        // Find matching groups
        let groups: Vec<Group> = db
            .query("SELECT name FROM group WHERE string::lowercase(name) CONTAINS $pattern")
            .bind(("pattern", pattern.clone()))
            .await?
            .take(0)?;

        Ok(SearchResults {
            hosts,
            tags,
            groups,
        })
    }

    /// Get hosts by selected criteria
    pub async fn get_filtered_hosts<C: Connection>(
        db: &Surreal<C>,
        selected_groups: Vec<String>,
        selected_tags: Vec<String>,
    ) -> Result<Vec<HostRecord>, Error> {
        let mut query = String::from("SELECT * FROM host WHERE 1=1");

        // Add group filters
        if !selected_groups.is_empty() {
            query.push_str(" AND id IN (SELECT VALUE out FROM groupped WHERE in IN (SELECT VALUE id FROM group WHERE name IN $groups))");
        }

        // Add tag filters
        if !selected_tags.is_empty() {
            query.push_str(" AND id IN (SELECT VALUE out FROM tagged WHERE in IN (SELECT VALUE id FROM tag WHERE name IN $tags))");
        }

        let hosts: Vec<HostRecord> = db
            .query(query)
            .bind(("groups", selected_groups))
            .bind(("tags", selected_tags))
            .await?
            .take(0)?;

        Ok(hosts)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::host::{
        table::{Group, Tag},
        EnhancedHost, Host,
    };
    use db::define_schema;
    // use serde::{Deserialize, Serialize};
    use surrealdb::engine::local::{Db, RocksDb};
    use tempdir::TempDir;

    async fn setup_test_data(db: &Surreal<Db>) -> Result<HostRecord, Error> {
        // Create tags
        let tag_abc = Tag::create(db, "abc".to_string()).await?;
        let tag_def = Tag::create(db, "def".to_string()).await?;

        // Create groups
        let group_dev = Group::create(db, "dev".to_string()).await?;
        let _group_prod = Group::create(db, "prod".to_string()).await?;
        let group_server = Group::create(db, "server".to_string()).await?;

        // Create hosts
        // Host A: has tag "def" and group "dev"
        let host_a = EnhancedHost {
            host: Host {
                name: "A".to_string(),
                bind_address: None,
                bind_interface: None,
                ca_signature_algorithms: None,
                certificate_file: None,
                ciphers: None,
                compression: None,
                connection_attempts: None,
                connect_timeout: None,
                host_key_algorithms: None,
                host_name: None,
                identity_file: None,
                ignore_unknown: None,
                kex_algorithms: None,
                mac: None,
                port: None,
                pubkey_accepted_algorithms: None,
                pubkey_authentication: None,
                remote_forward: None,
                server_alive_interval: None,
                tcp_keep_alive: None,
                user: None,
                proxy_jump: None,
                ignored_fields: HashMap::new(),
                unsupported_fields: HashMap::new(),
                #[cfg(target_os = "macos")]
                use_keychain: None,
            },
            comment: None,
        };
        let host_a_record = EnhancedHost::create(db, host_a).await?;
        EnhancedHost::add_tag(db, &host_a_record.id, &tag_def).await?;
        EnhancedHost::add_group(db, &host_a_record.id, &group_dev).await?;

        // Host B: only in group "dev"
        let host_b = EnhancedHost {
            host: Host {
                name: "B".to_string(),
                bind_address: None,
                bind_interface: None,
                ca_signature_algorithms: None,
                certificate_file: None,
                ciphers: None,
                compression: None,
                connection_attempts: None,
                connect_timeout: None,
                host_key_algorithms: None,
                host_name: None,
                identity_file: None,
                ignore_unknown: None,
                kex_algorithms: None,
                mac: None,
                port: None,
                pubkey_accepted_algorithms: None,
                pubkey_authentication: None,
                remote_forward: None,
                server_alive_interval: None,
                tcp_keep_alive: None,
                user: None,
                proxy_jump: None,
                ignored_fields: HashMap::new(),
                unsupported_fields: HashMap::new(),
                #[cfg(target_os = "macos")]
                use_keychain: None,
            },
            comment: None,
        };
        let host_b_record = EnhancedHost::create(db, host_b).await?;
        EnhancedHost::add_group(db, &host_b_record.id, &group_dev).await?;

        // Host D: has both tags and group "server"
        let host_d = EnhancedHost {
            host: Host {
                name: "D".to_string(),
                bind_address: None,
                bind_interface: None,
                ca_signature_algorithms: None,
                certificate_file: None,
                ciphers: None,
                compression: None,
                connection_attempts: None,
                connect_timeout: None,
                host_key_algorithms: None,
                host_name: None,
                identity_file: None,
                ignore_unknown: None,
                kex_algorithms: None,
                mac: None,
                port: None,
                pubkey_accepted_algorithms: None,
                pubkey_authentication: None,
                remote_forward: None,
                server_alive_interval: None,
                tcp_keep_alive: None,
                user: None,
                proxy_jump: None,
                ignored_fields: HashMap::new(),
                unsupported_fields: HashMap::new(),
                #[cfg(target_os = "macos")]
                use_keychain: None,
            },
            comment: None,
        };
        let host_d_record = EnhancedHost::create(db, host_d).await?;
        EnhancedHost::add_tag(db, &host_d_record.id, &tag_abc).await?;
        EnhancedHost::add_tag(db, &host_d_record.id, &tag_def).await?;
        EnhancedHost::add_group(db, &host_d_record.id, &group_server).await?;

        Ok(host_d_record)
    }

    #[tokio::test]
    async fn test_search_flow() -> Result<(), Error> {
        let temp_dir = TempDir::new("db").unwrap();
        let db = Surreal::new::<RocksDb>(temp_dir.path()).await.unwrap();
        let _ = db.use_ns("test").use_db("test").await;
        let _ = define_schema(&db).await;

        // Setup test data
        let host_d = setup_test_data(&db).await?;
        let tag: Tag = Tag::new("def".to_string());
        let group: Group = Group::new("dev".to_string());

        // When user types "d"
        let suggestions = HostSearch::suggest(&db, "d").await?;
        assert!(suggestions.hosts.contains(&host_d));
        assert!(suggestions.tags.contains(&tag));
        assert_eq!(suggestions.groups.len(), 2);
        assert!(suggestions.groups.contains(&group));

        let suggestions = HostSearch::suggest(&db, "dE").await?;
        assert_eq!(suggestions.hosts.len(), 0);
        assert!(suggestions.tags.contains(&tag));
        assert_eq!(suggestions.groups.len(), 1);
        assert!(suggestions.groups.contains(&group));

        // // When user selects group "dev"
        let hosts = HostSearch::get_filtered_hosts(&db, vec!["dev".to_string()], vec![]).await?;
        // // Should contain hosts A and B
        assert_eq!(hosts.len(), 2);
        assert!(hosts.iter().any(|h| h.host.name == "A"));
        assert!(hosts.iter().any(|h| h.host.name == "B"));

        Ok(())
    }
}
