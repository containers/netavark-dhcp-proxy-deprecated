# Netavark DHCP Proxy Server

### Short Summary

Adding the macvlan functionality to Podman’s new network stack

### Objective

The old network stack that used container-networking-plugins (CNI) had the ability to configure a macvlan network.  The primary purpose of the macvlan set up was to provide the container with a routable IP address so Network Address Translation (NAT) and port mapping was not needed. This functionality has not been added to the new network stack (netavark) yet.

One of the challenges of the macvlan set up is that containers do not typically have a full set of network tools installed in them.  And most certainly, very few have things like dhclient. And even if they did, they will usually lack an init system that could deal with re-leasing IP addresses.  The solution for CNI was to create a separate binary that acted as a “dhcp-proxy” for macvlan containers.  The dhcp-proxy was generally run by systemd and it performed the DHCP operations needed by the container.  This lease information that was then provided by the DHCP server was then statically assigned to the container by Podman.

For the new implementation, we will largely follow the same architecture but will attempt to provide improvements as needed.

### Use case

I have a web-based service that runs from a container.  While I know I can expose the service from the container host using port-mapping, I do not want users to have to specify any port related information when communicating with the web service.  I want the container to appear as if it were any other host on the network to its users.

![topo](img/topo.png)

## Build
```cargo build```

## Run
Server
```cargo run --bin server```

Client
```cargo run --bin client```

## Test

```cargo test```