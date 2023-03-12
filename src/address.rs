use ipnet::IpNet;
use std::net::IpAddr;
use std::net::ToSocketAddrs;
use std::str::FromStr;

pub fn expand_local_address(address_family: &str, address: &Option<String>) -> Vec<String> {
    if let Some(address) = address {
        if let Ok(address) = IpAddr::from_str(address) {
            // TODO: check if address family matches
            return vec![address.to_string()];
        }
        if let Ok(cidr) = IpNet::from_str(address) {
            // TODO: check if address family matches
            return vec![cidr.to_string()];
        }
        vec![]
    } else {
        match address_family {
            "ip4" => vec!["0.0.0.0/0".to_string()],
            "ip6" => vec!["::/0".to_string()],
            _ => vec![],
        }
    }
}

pub fn expand_remote_address(address_family: &str, address: &Option<String>) -> Vec<String> {
    let mut addresses = match address_family {
        "ip4" => vec!["0.0.0.0/0".to_string()],
        "ip6" => vec!["::/0".to_string()],
        _ => vec![],
    };

    if let Some(address) = address {
        if let Some(address) = (address.as_str(), 0)
            .to_socket_addrs()
            .unwrap_or_else(|_| vec![].into_iter())
            .find(|addr| match address_family {
                "ip4" => addr.is_ipv4(),
                "ip6" => addr.is_ipv6(),
                _ => false,
            })
            .map(|addr| addr.ip().to_string())
        {
            addresses.push(address);
        }
    }

    addresses
}

#[cfg(test)]
mod test {
    #[test]
    fn expand_local_address() {
        assert_eq!(
            super::expand_local_address("invalid", &None),
            Vec::<String>::new()
        );
        assert_eq!(super::expand_local_address("ip4", &None), vec!["0.0.0.0/0"]);
        assert_eq!(super::expand_local_address("ip6", &None), vec!["::/0"]);
        assert_eq!(
            super::expand_local_address("ip4", &Some("127.0.0.1".to_string())),
            vec!["127.0.0.1"]
        );
        assert_eq!(
            super::expand_local_address("ip6", &Some("::1".to_string())),
            vec!["::1"]
        );
        assert_eq!(
            super::expand_local_address("ip4", &Some("10.0.0.0/24".to_string())),
            vec!["10.0.0.0/24"]
        );
        assert_eq!(
            super::expand_local_address("ip6", &Some("fd00::/8".to_string())),
            vec!["fd00::/8"]
        );
    }

    #[test]
    fn expand_remote_address() {
        assert_eq!(
            super::expand_remote_address("invalid", &None),
            Vec::<String>::new()
        );
        assert_eq!(
            super::expand_remote_address("ip4", &None),
            vec!["0.0.0.0/0".to_string()]
        );
        assert_eq!(
            super::expand_remote_address("ip6", &None),
            vec!["::/0".to_string()]
        );
        assert_eq!(
            super::expand_remote_address("ip4", &Some("name.invalid".to_string())),
            vec!["0.0.0.0/0".to_string()]
        );
        assert_eq!(
            super::expand_remote_address("ip6", &Some("name.invalid".to_string())),
            vec!["::/0".to_string()]
        );
        assert_eq!(
            super::expand_remote_address("ip4", &Some("localhost".to_string())),
            vec!["0.0.0.0/0".to_string(), "127.0.0.1".to_string()]
        );
        assert_eq!(
            super::expand_remote_address("ip6", &Some("localhost".to_string())),
            vec!["::/0".to_string(), "::1".to_string()]
        );
    }
}
