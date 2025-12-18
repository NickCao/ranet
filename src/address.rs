use ipnet::IpNet;
use std::net::IpAddr;
use std::net::ToSocketAddrs;
use std::str::FromStr;

pub fn local(address_family: &str, address: &Option<String>) -> Vec<String> {
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

pub fn remote(address_family: &str, address: &Option<String>) -> Vec<String> {
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
    fn local() {
        insta::assert_yaml_snapshot!(super::local("invalid", &None));
        insta::assert_yaml_snapshot!(super::local("ip4", &None));
        insta::assert_yaml_snapshot!(super::local("ip6", &None));
        insta::assert_yaml_snapshot!(super::local("ip4", &Some("127.0.0.1".to_string())),);
        insta::assert_yaml_snapshot!(super::local("ip6", &Some("::1".to_string())));
        insta::assert_yaml_snapshot!(super::local("ip4", &Some("10.0.0.0/24".to_string())),);
        insta::assert_yaml_snapshot!(super::local("ip6", &Some("fd00::/8".to_string())),);
    }

    #[test]
    fn remote() {
        insta::assert_yaml_snapshot!(super::remote("invalid", &None));
        insta::assert_yaml_snapshot!(super::remote("ip4", &None));
        insta::assert_yaml_snapshot!(super::remote("ip6", &None));
        insta::assert_yaml_snapshot!(super::remote("ip4", &Some("name.invalid".to_string())),);
        insta::assert_yaml_snapshot!(super::remote("ip6", &Some("name.invalid".to_string())),);
        insta::assert_yaml_snapshot!(super::remote("ip4", &Some("localhost".to_string())),);
        insta::assert_yaml_snapshot!(super::remote("ip6", &Some("localhost".to_string())),);
    }
}
