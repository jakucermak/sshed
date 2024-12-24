pub mod network {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[derive(Debug, Clone, PartialEq)]
    pub enum NetworkAddress {
        IPv4(Ipv4Addr),
        IPv6(Ipv6Addr),
        Hostname(String),
    }

    impl NetworkAddress {
        pub fn new_ipv4(addr: Ipv4Addr) -> Self {
            NetworkAddress::IPv4(addr)
        }

        pub fn new_ipv6(addr: Ipv6Addr) -> Self {
            NetworkAddress::IPv6(addr)
        }

        pub fn new_hostname(hostname: String) -> Self {
            NetworkAddress::Hostname(hostname)
        }

        pub fn to_string(&self) -> String {
            match self {
                NetworkAddress::IPv4(addr) => addr.to_string(),
                NetworkAddress::IPv6(addr) => addr.to_string(),
                NetworkAddress::Hostname(hostname) => hostname.clone(),
            }
        }
    }
}
