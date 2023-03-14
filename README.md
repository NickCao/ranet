## ranet - redundant array of networks

#### About

ranet, acronym for redundant array of networks, forms full mesh IPSec connectivity among network participants.

#### Configuration

```json5
{
  "organization": "acme corp", // unique identifier of a keypair
  "common_name": "some server", // node name, unique within an organization
  "endpoints": [
    {
      "serial_number": "0", // fort distinguishing endpoints, unique within a node
      "address": "1.1.1.1", // ip address or cidr, can be omitted
      "address_family": "ip4", // or ip6
      "port": 13000, // must be identical to charon.port_nat_t
      "updown": "/usr/local/bin/updown", // script to run on connection changes, see https://docs.strongswan.org/docs/5.9/plugins/updown.html
      "fwmark": null // see <child>.set_mark_out in https://docs.strongswan.org/docs/5.9/swanctl/swanctlConf.html
    },
    {
      "serial_number": "1",
      "address": null,
      "address_family": "ip6",
      "port": 13000,
      "updown": "/usr/local/bin/updown",
      "fwmark": null
    }
  ]
}
```

#### Registry

The registry is a json array containing information of mesh participants.
```json5
[
  {
    "public_key": "<PEM encoded public key>",
    "organization": "acme corp",
    "nodes": [
      {
        "common_name": "some server",
        "endpoints": [
          {
            "serial_number": "0", // matches one-to-one with endpoints in local config
            "address_family": "ip4",
            "address": "example.com", // ip or domain name, can be omitted
            "port": 13000
          },
          {
            "serial_number": "1",
            "address_family": "ip6",
            "address": null,
            "port": 13000
          }
        ],
        "remarks": {
          "arbitrary": "metadata"
        }
      }
    ]
  }
]
```
