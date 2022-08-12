# Netavark DHCP Proxy Server

This binary crate aims to automate the macvlan dhcp issue for netavark. This binary should be 
able to make DHCP DORA requests on behalf of netavark. Note that this is not a DHCP server, rather
a proxy to communicate with netavark network config and DHCP server. This is the main [issue](https://github.com/containers/netavark/issues/152).

### Task List
- [x] Setup g_rpc sever - [tonic](https://github.com/hyperium/tonic) 
- [ ] DHCP DORA with event listener using [mozim](https://github.com/nispor/mozim)
- [ ] Call back netavark if network config needs to change
- [ ] Enable DHCPv6 DORA




 