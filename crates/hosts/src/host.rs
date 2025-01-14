pub mod table;
use std::{collections::HashMap, ops::Index, path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};
use ssh2_config::HostParams;
use surrealdb::{sql::Thing, Connection, Error, Surreal};
use table::{Group, Tag};

#[derive(Debug, Deserialize, Serialize)]
pub struct Host {
    /// Hosts name in file.
    pub name: String,
    /// Specifies to use the specified address on the local machine as the source address of the connection
    pub bind_address: Option<String>,
    /// Use the specified address on the local machine as the source address of the connection
    pub bind_interface: Option<String>,
    /// Specifies which algorithms are allowed for signing of certificates by certificate authorities
    pub ca_signature_algorithms: Option<Vec<String>>,
    /// Specifies a file from which the user's certificate is read
    pub certificate_file: Option<PathBuf>,
    /// Specifies the ciphers allowed for protocol version 2 in order of preference
    pub ciphers: Option<Vec<String>>,
    /// Specifies whether to use compression
    pub compression: Option<bool>,
    /// Specifies the number of attempts to make before exiting
    pub connection_attempts: Option<usize>,
    /// Specifies the timeout used when connecting to the SSH server
    pub connect_timeout: Option<Duration>,
    /// Specifies the host key signature algorithms that the client wants to use in order of preference
    pub host_key_algorithms: Option<Vec<String>>,
    /// Specifies the real host name to log into
    pub host_name: Option<String>,
    /// Specifies the path of the identity file to be used when authenticating.
    /// More than one file can be specified.
    /// If more than one file is specified, they will be read in order
    pub identity_file: Option<Vec<PathBuf>>,
    /// Specifies a pattern-list of unknown options to be ignored if they are encountered in configuration parsing
    pub ignore_unknown: Option<Vec<String>>,
    /// Specifies the available KEX (Key Exchange) algorithms
    pub kex_algorithms: Option<Vec<String>>,
    /// Specifies the MAC (message authentication code) algorithms in order of preference
    pub mac: Option<Vec<String>>,
    /// Specifies the port number to connect on the remote host.
    pub port: Option<u16>,
    /// Specifies the signature algorithms that will be used for public key authentication
    pub pubkey_accepted_algorithms: Option<Vec<String>>,
    /// Specifies whether to try public key authentication using SSH keys
    pub pubkey_authentication: Option<bool>,
    /// Specifies that a TCP port on the remote machine be forwarded over the secure channel
    pub remote_forward: Option<u16>,
    /// Sets a timeout interval in seconds after which if no data has been received from the server, keep alive will be sent
    pub server_alive_interval: Option<Duration>,
    /// Specifies whether to send TCP keepalives to the other side
    pub tcp_keep_alive: Option<bool>,
    #[cfg(target_os = "macos")]
    /// specifies whether the system should search for passphrases in the user's keychain when attempting to use a particular key
    pub use_keychain: Option<bool>,
    /// Specifies the user to log in as.
    pub user: Option<String>,
    pub proxy_jump: Option<Vec<String>>,
    /// fields that the parser wasn't able to parse
    pub ignored_fields: HashMap<String, Vec<String>>,
    /// fields that the parser was able to parse but ignored
    pub unsupported_fields: HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedHost {
    pub host: Host,
    pub comment: Option<String>,
}

impl EnhancedHost {
    pub async fn create<C: Connection>(
        db: &Surreal<C>,
        data: EnhancedHost,
    ) -> surrealdb::Result<Thing> {
        let host_name = data.host.name.clone();
        let created: Option<EnhancedHost> = db.create("host").content(data).await?;

        let _ = created.ok_or(Error::Db(surrealdb::error::Db::InvalidModel {
            message: String::from("Failed to create new host"),
        }))?;

        let record_id = match Self::get_id_by_name(db, host_name).await {
            Ok(t) => match t {
                Some(t) => t,
                None => todo!(),
            },
            Err(e) => return Err(e),
        };

        Ok(record_id)
    }

    pub async fn get_id_by_name<C: Connection>(
        db: &Surreal<C>,
        name: String,
    ) -> surrealdb::Result<Option<Thing>> {
        #[derive(Debug, Deserialize)]
        struct Record {
            id: Thing,
        }

        let record_id: Option<Record> = db
            .query("SELECT id FROM host WHERE host.name = $name LIMIT 1")
            .bind(("name", name.clone()))
            .await
            .unwrap()
            .take(0)?;

        match record_id {
            Some(r) => Ok(Some(r.id)),
            None => Ok(None),
        }
    }

    pub async fn update<C: Connection>(db: &Surreal<C>, data: Self) -> Result<Thing, Error> {
        let record_id = match Self::get_id_by_name(&db, data.host.name.clone()).await {
            Ok(t) => match t {
                Some(t) => t,
                None => todo!(),
            },
            Err(e) => return Err(e),
        };

        let updated: Option<EnhancedHost> = db
            .update(("host", &record_id.id.to_string()))
            .content(data)
            .await?;

        let _ = updated.ok_or(Error::Db(surrealdb::error::Db::InvalidModel {
            message: String::from("Failed to create new host"),
        }))?;

        Ok(record_id)
    }

    pub async fn create_or_update<C: Connection>(
        db: &Surreal<C>,
        data: EnhancedHost,
    ) -> surrealdb::Result<Thing> {
        let existing: Option<Thing> = Self::get_id_by_name(db, data.host.name.clone())
            .await
            .unwrap();

        match existing {
            Some(_) => {
                let updated: surrealdb::Result<Thing> = Self::update(db, data).await;
                updated
            }
            None => {
                let created: surrealdb::Result<Thing> = Self::create(db, data).await;
                created
            }
        }
    }

    pub async fn add_tag<C: Connection>(
        db: &Surreal<C>,
        host_id: &Thing,
        tag_id: &Thing,
    ) -> Result<(), Error> {
        Tag::add_relation(&db, host_id, tag_id).await
    }

    pub async fn remove_tag<C: Connection>(
        db: &Surreal<C>,
        host_id: &Thing,
        tag_id: &Thing,
    ) -> Result<(), Error> {
        Tag::remove_relation(db, host_id, tag_id).await
    }

    pub async fn add_group<C: Connection>(
        db: &Surreal<C>,
        host_id: &Thing,
        group_id: &Thing,
    ) -> Result<(), Error> {
        Group::add_relation(&db, host_id, group_id).await
    }

    pub async fn remove_group<C: Connection>(
        db: &Surreal<C>,
        host_id: &Thing,
        group_id: &Thing,
    ) -> Result<(), Error> {
        Group::remove_relation(db, host_id, group_id).await
    }

    pub async fn get_tags<C: Connection>(
        db: &Surreal<C>,
        host_id: &Thing,
    ) -> Result<HashMap<Thing, Tag>, Error> {
        let record_ids: Vec<Thing> = match db
            .query("RETURN $host <-tagged<-tag")
            .bind(("host", host_id.clone()))
            .await
        {
            Ok(mut r) => match r.take(0) {
                Ok(t) => t,
                Err(e) => {
                    return Err(e);
                }
            },
            Err(e) => return Err(e),
        };

        let mut hm: HashMap<Thing, Tag> = HashMap::new();

        for tag_id in record_ids {
            let tag: Option<Tag> = db
                .query("SELECT * FROM tag WHERE id = $tid LIMIT 1")
                .bind(("tid", tag_id.clone()))
                .await
                .unwrap()
                .take(0)?;

            match tag {
                Some(t) => {
                    let _ = hm.insert(tag_id, t);
                }
                None => todo!(),
            }
        }

        Ok(hm)
    }

    pub async fn get_groups<C: Connection>(
        db: &Surreal<C>,
        host_id: &Thing,
    ) -> Result<HashMap<Thing, Group>, Error> {
        let record_ids: Vec<Thing> = match db
            .query("RETURN $host <-groupped<-group")
            .bind(("host", host_id.clone()))
            .await
        {
            Ok(mut r) => match r.take(0) {
                Ok(t) => t,
                Err(e) => {
                    return Err(e);
                }
            },
            Err(e) => return Err(e),
        };

        let mut hm: HashMap<Thing, Group> = HashMap::new();

        for group_id in record_ids {
            let group: Option<Group> = db
                .query("SELECT * FROM group WHERE id = $gid LIMIT 1")
                .bind(("gid", group_id.clone()))
                .await
                .unwrap()
                .take(0)?;

            match group {
                Some(t) => {
                    let _ = hm.insert(group_id, t);
                }
                None => todo!(),
            }
        }

        Ok(hm)
    }
}

impl From<ssh2_config::Host> for Host {
    fn from(host: ssh2_config::Host) -> Self {
        let params: HostParams = host.params;
        let pattern = host.pattern.index(0).pattern.clone();

        Self {
            name: pattern,
            bind_address: params.bind_address,
            bind_interface: params.bind_interface,
            ca_signature_algorithms: params.ca_signature_algorithms,
            certificate_file: params.certificate_file,
            ciphers: params.ciphers,
            compression: params.compression,
            connection_attempts: params.connection_attempts,
            connect_timeout: params.connect_timeout,
            host_key_algorithms: params.host_key_algorithms,
            host_name: params.host_name,
            identity_file: params.identity_file,
            ignore_unknown: params.ignore_unknown,
            kex_algorithms: params.kex_algorithms,
            mac: params.mac,
            port: params.port,
            pubkey_accepted_algorithms: params.pubkey_accepted_algorithms,
            pubkey_authentication: params.pubkey_authentication,
            remote_forward: params.remote_forward,
            server_alive_interval: params.server_alive_interval,
            tcp_keep_alive: params.tcp_keep_alive,
            #[cfg(target_os = "macos")]
            use_keychain: params.use_keychain,
            user: params.user,
            ignored_fields: params.ignored_fields,
            unsupported_fields: params.unsupported_fields,
            proxy_jump: params.proxy_jump,
        }
    }
}

#[cfg(test)]
mod tests {}
