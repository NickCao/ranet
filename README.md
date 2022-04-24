## ranet - redundant array of network

#### About

ranet, acronym for redundant array of network, is the missing piece of the puzzle, for using wireguard to create distributed overlay networks. It serves the purpose by creating point to point tunnels between all participants, forming fully-meshed link-local connectivity. This is actually a rewrite of a previous project, [RAIT](https://gitlab.com/NickCao/RAIT), to address some shortcomings in its design, and, RIIR.

#### Design

Due to technical limitation of wireguard, namely crypto routing, it struggles to be integrated into routing daemons, thus we takes a different approach, creating a separate interface for each peer, *abusing* wireguard as a point to point transport, opposing to it's original design. While this approach do ensures fully-meshed connectivity instead of a hub and spoke architecture, it also voids the possibility to reuse a single port for multiple peers, though the consumption of port range is negligible (after all, we have 65535 ports to waste), the coordination of port usage is a challenging task. Ranet solves the problem with the concept of "send port", a unique port assigned to each node, as the destination port of all packets originated by it. To separate overlay from underlay, instead of using netns as in RAIT, VRF is used for it's simplicity and flexibility.

