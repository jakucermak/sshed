use address::network::NetworkAddress;
use tag::tag::Tag;

pub mod address;
pub mod tag;

trait HostManager {
    fn get_port(port: Option<u16>) -> u16 {
        match port {
            Some(v) => v,
            None => 22,
        }
    }

    fn new(
        name: Option<String>,
        address: NetworkAddress,
        port: Option<u16>,
        username: Option<String>,
        identity_file: Option<String>,
        tags: Vec<Tag>,
    ) -> Host;

    fn update(
        &mut self,
        name: Option<String>,
        address: Option<NetworkAddress>,
        port: Option<u16>,
        username: Option<String>,
        identity_file: Option<String>,
        tags: Option<Vec<Tag>>,
    );
}

/// Todo: Implement storing password in hash
#[derive(Debug)]
struct Host {
    name: String,
    address: NetworkAddress,
    port: u16,
    username: Option<String>,
    identity_file: Option<String>,
    tags: Vec<Tag>,
}

impl HostManager for Host {
    fn new(
        name: Option<String>,
        address: NetworkAddress,
        port: Option<u16>,
        username: Option<String>,
        identity_file: Option<String>,
        tags: Vec<Tag>,
    ) -> Host {
        let name = match name {
            Some(n) => n,
            None => address.to_string(),
        };

        Self {
            name,
            address,
            port: Self::get_port(port),
            username,
            identity_file,
            tags,
        }
    }

    fn update(
        &mut self,
        name: Option<String>,
        address: Option<NetworkAddress>,
        port: Option<u16>,
        username: Option<String>,
        identity_file: Option<String>,
        tags: Option<Vec<Tag>>,
    ) {
        if let Some(n) = name {
            self.name = n;
        }
        if let Some(a) = address {
            self.address = a;
        }
        if let Some(p) = port {
            self.port = Self::get_port(Some(p));
        }
        if let Some(u) = username {
            self.username = Some(u);
        }
        if let Some(i) = identity_file {
            self.identity_file = Some(i);
        }
        if let Some(t) = tags {
            self.tags = t;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::*;

    #[test]
    fn port_provided() {
        let h = Host::new(
            None,
            NetworkAddress::new_ipv4(Ipv4Addr::new(127, 0, 0, 1)),
            Some(2222),
            None,
            None,
            Vec::new(),
        );
        assert_eq!(h.port, 2222);
    }

    #[test]
    fn port_not_provided() {
        let h = Host::new(
            None,
            NetworkAddress::new_ipv4(Ipv4Addr::new(127, 0, 0, 1)),
            None,
            None,
            None,
            Vec::new(),
        );
        assert_eq!(h.port, 22);
    }

    #[test]
    fn name_not_provided() {
        let h = Host::new(
            None,
            NetworkAddress::new_ipv4(Ipv4Addr::new(127, 0, 0, 1)),
            None,
            None,
            None,
            Vec::new(),
        );
        assert_eq!(h.name, "127.0.0.1")
    }

    #[test]
    fn name_provided() {
        let h = Host::new(
            Some("Supercool Host with Power".to_string()),
            NetworkAddress::new_ipv4(Ipv4Addr::new(127, 0, 0, 1)),
            None,
            None,
            None,
            Vec::new(),
        );
        assert_eq!(h.name, "Supercool Host with Power")
    }

    #[test]
    fn update_name() {
        let mut h = Host::new(
            None,
            NetworkAddress::new_ipv4(Ipv4Addr::new(127, 0, 0, 1)),
            None,
            None,
            None,
            Vec::new(),
        );
        h.update(
            Some("Updated Name".to_string()),
            None,
            None,
            None,
            None,
            None,
        );
        assert_eq!(h.name, "Updated Name");
    }

    #[test]
    fn update_address() {
        let mut h = Host::new(
            None,
            NetworkAddress::new_ipv4(Ipv4Addr::new(127, 0, 0, 1)),
            None,
            None,
            None,
            Vec::new(),
        );
        h.update(
            None,
            Some(NetworkAddress::new_ipv4(Ipv4Addr::new(192, 168, 1, 1))),
            None,
            None,
            None,
            None,
        );
        assert_eq!(h.address.to_string(), "192.168.1.1");
    }

    #[test]
    fn update_port() {
        let mut h = Host::new(
            None,
            NetworkAddress::new_ipv4(Ipv4Addr::new(127, 0, 0, 1)),
            None,
            None,
            None,
            Vec::new(),
        );
        h.update(None, None, Some(2222), None, None, None);
        assert_eq!(h.port, 2222);
    }

    #[test]
    fn update_username() {
        let mut h = Host::new(
            None,
            NetworkAddress::new_ipv4(Ipv4Addr::new(127, 0, 0, 1)),
            None,
            None,
            None,
            Vec::new(),
        );
        h.update(None, None, None, Some("newuser".to_string()), None, None);
        assert_eq!(h.username, Some("newuser".to_string()));
    }

    #[test]
    fn update_identity_file() {
        let mut h = Host::new(
            None,
            NetworkAddress::new_ipv4(Ipv4Addr::new(127, 0, 0, 1)),
            None,
            None,
            None,
            Vec::new(),
        );
        h.update(
            None,
            None,
            None,
            None,
            Some("/path/to/identity".to_string()),
            None,
        );
        assert_eq!(h.identity_file, Some("/path/to/identity".to_string()));
    }

    #[test]
    fn update_tags() {
        let mut h = Host::new(
            None,
            NetworkAddress::new_ipv4(Ipv4Addr::new(127, 0, 0, 1)),
            None,
            None,
            None,
            Vec::new(),
        );
        let new_tags = vec![Tag::new("test".to_string())];
        h.update(None, None, None, None, None, Some(new_tags));
        assert_eq!(h.tags[0].get_name(), "test");
    }
}
