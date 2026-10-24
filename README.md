## ranet - redundant array of networks

#### About

ranet, acronym for redundant array of networks, forms full mesh IPSec connectivity among network participants.

#### XFRM interface IDs

Each connection uses the same numeric `if_id_in` and `if_id_out`, derived from
SHA-256 of its full local and remote endpoint identities. Addresses, ports,
public keys, and registry ordering do not form part of the hash input.

Connections are processed in a canonical order. If an ID is occupied, ranet
assigns the next free ID, wrapping around and skipping zero and strongSwan's
reserved `%unique`/`%unique-dir` values. The same identity set reconstructs the
same assignments without a state file. Adding or removing colliding identities
can change the IDs assigned within that collision group.

New IKE_SAs use these IDs; existing SAs retain their IDs until re-established.

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
  ],
  "experimental": {
    "trap": false // set true to install trap policies and skip immediate initiation
  }
}
```

`experimental.trap` applies to all connections and defaults to `false`, keeping
immediate initiation. When enabled, ranet loads connections with
`start_action = trap` and skips immediate initiation. Trap policies and active
SAs share the stable XFRM interface ID described above. Automatic restart after
DPD is enabled in both modes.

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
