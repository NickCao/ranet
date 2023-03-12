use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Registry = Vec<Organization>;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Organization {
    pub public_key: String,
    pub organization: String,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub common_name: String,
    pub endpoints: Vec<Endpoint>,
    pub remarks: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Endpoint {
    pub serial_number: String,
    pub address_family: String,
    pub address: Option<String>,
    pub port: u16,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deserialize() {
        let data = r#"
            [
              {
                "public_key": "<PEM encoded public key>",
                "organization": "nickcao",
                "nodes": [
                  {
                    "common_name": "nrt0",
                    "endpoints": [
                      {
                        "serial_number": "0",
                        "address_family": "ip4",
                        "port": 3000
                      },
                      {
                        "serial_number": "1",
                        "address_family": "ip6",
                        "address": "nrt0.nichi.link",
                        "port": 4000
                      }
                    ],
                    "remarks": {
                      "some": "random note"
                    }
                  }
                ]
              }
            ]
        "#;

        let value: Registry = serde_json::from_str(data).unwrap();

        assert_eq!(
            value,
            vec![Organization {
                public_key: "<PEM encoded public key>".to_string(),
                organization: "nickcao".to_string(),
                nodes: vec![Node {
                    common_name: "nrt0".to_string(),
                    endpoints: vec![
                        Endpoint {
                            serial_number: "0".to_string(),
                            address_family: "ip4".to_string(),
                            address: None,
                            port: 3000
                        },
                        Endpoint {
                            serial_number: "1".to_string(),
                            address_family: "ip6".to_string(),
                            address: Some("nrt0.nichi.link".to_string()),
                            port: 4000
                        }
                    ],
                    remarks: HashMap::from([("some".to_string(), "random note".to_string())])
                }]
            }]
        )
    }
}
