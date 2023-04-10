# Netavark DHCP Proxy Server

## This prroject is now deprecated.  It has been merged with [netavark](https://github.com/containers/netavark)


### Short Summary

Adds DHCP MacVLAN functionality to the [netavark](https://github.com/containers/netavark) networking stack.  It's primary
use is with Podman.

### Objective

The old network stack that used container-networking-plugins (CNI) had the ability to configure a macvlan network.  The primary purpose of the macvlan set up was to provide the container with a routable IP address so Network Address Translation (NAT) and port mapping was not needed. This functionality has not been added to the new network stack (netavark) yet.

One of the challenges of the macvlan set up is that containers do not typically have a full set of network tools installed in them.  And most certainly, very few have things like dhclient. And even if they did, they will usually lack an init system that could deal with re-leasing IP addresses.  The solution for CNI was to create a separate binary that acted as a “dhcp-proxy” for macvlan containers.  The dhcp-proxy was generally run by systemd,
and it performed the DHCP lease operations needed by the container.  This lease information that was then provided by the DHCP server was then statically assigned to the container by Podman.

For the new implementation, we will largely follow the same architecture but will attempt to provide improvements as needed.

### Use case

I have a web-based service that runs from a container.  While I know I can expose the service from the container host using port-mapping, I do not want users to have to specify any port related information when communicating with the web service.  I want the container to appear as if it were any other host on the network to its users.

![topo](img/topo.png)

## Build dependencies
This package is written in rust and therefore requires a rust toolchain.  In addition to the toolchain, the following
dependencies are also applicable:

* make
* protobuf-compiler
* proto-c
* gcc

### Fedora

```
$ sudo dnf install protobuf-compiler protobuf-c make gcc
```

### Ubuntu
```
$ sudo apt-get install make gcc protobuf-c-compiler protobuf-compiler
```

## Build
To build both the proxy server and the test client
```
$ make all
```

## Testing
```

You can run make test to run both unit and integration tests.  There are also make targets
for `unit` and `integration` that can be run separately.

$ make test
```
