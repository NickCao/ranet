## ranet - redundant array of network

#### About

ranet, acronym for redundant array of network, is the missing piece of the puzzle, for using wireguard to create distributed overlay networks. It serves the purpose by creating point to point tunnels between all participants, forming fully-meshed link-local connectivity. This is actually a rewrite of a previous project, [RAIT](https://gitlab.com/NickCao/RAIT), to address some shortcomings in its design, and, RIIR.

#### Design

Due to technical limitation of wireguard, namely crypto routing, it struggles to be integrated into routing daemons, thus we takes a different approach, creating a separate interface for each peer, *abusing* wireguard as a point to point transport, opposing to it's original design. While this approach do ensures fully-meshed connectivity instead of a hub and spoke architecture, it also voids the possibility to reuse a single port for multiple peers, though the consumption of port range is negligible (after all, we have 65535 ports to waste), the coordination of port usage is a challenging task. Ranet solves the problem with the concept of "send port", a unique port assigned to each node, as the destination port of all packets originated by it. To separate overlay from underlay, instead of using netns as in RAIT, VRF is used for it's simplicity and flexibility. Tracking the creation and destruction of a large number of network interfaces is accomplished with interface groups, a numeric value attached to every interface on linux. When creating interfaces, all existing interfaces are marked as stale, then both newly created and reconfigured interfaces are marked as active, finally, all stale interfaces are removed, this technique effectively offloads the boring accounting to the kernel.

#### Configuration

**The configuration format is subject to change.**
```json5
{
  "registry": "/etc/ranet/registry.json", // path to registry file
  "private_key": "KOfdX54OtHAOMqSn2rytCDHSFNe8prw43O8A98z21Xg=", // wireguard private key
  "vrf": "ranet", // name of the VRF enslaving wireguard interfaces
  "stale_group": 1, // group id for stale interfaces
  "active_group": 2, // group id for active interfaces
  // useful for dual stack underlay network or multihoming scenarios
  "transport": [
    {
      "address_family": "ip4", // address family, ip4 or ip6
      "address": "example.com", // public address for inbound connection, can be left empty
      "send_port": 50153, // send port, MUST be unique across the whole mesh
      "mtu": 1400, // interface mtu
      "prefix": "ranet4x", // interface name prefix
      "fwmark": 54, // wireguard fwmark
      "random_port": false // whether to listen on random ports
    },
    {
      "address_family": "ip6",
      "address": "example.com",
      "send_port": 50154,
      "mtu": 1400,
      "prefix": "ranet6x",
      "fwmark": 54,
      "random_port": false
    }
  ],
  // arbitrary key value peers for humans
  "remarks" : {
    "location": "earth",
    "operator": "human"
  }
}
```
