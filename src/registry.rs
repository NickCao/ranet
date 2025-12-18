use serde::{de::IgnoredAny, Deserialize, Serialize};

pub type Registry = Vec<Organization>;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Organization {
    pub public_key: String,
    pub organization: String,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub common_name: String,
    pub endpoints: Vec<Endpoint>,
    #[serde(default, skip_serializing)]
    pub remarks: Option<IgnoredAny>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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
                      "some": "random note",
                      "other": false
                    }
                  }
                ]
              }
            ]
        "#;

        let value: Registry = serde_json::from_str(data).unwrap();

        insta::assert_yaml_snapshot!(value);
    }
}
