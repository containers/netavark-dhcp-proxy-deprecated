% netavark-dhcp-proxy(1)

## NAME
netavark-dhcp-proxy - a proxy for DHCP interactions with containers

## SYNOPSIS
**netavark-dhcp-proxy** [*options*] *command* <config>

## DESCRIPTION
When using DHCP with MacVLAN and containers, you need the container to either have
an init system with DHCP clients or some sort of proxy server that can act on behalf
of the container.  The netavark-dhcp-proxy is the latter and should be used in
combination with Podman and Netavark when setting up containers that wish to use
DHCP and MacVLAN networking.

**netavark-dhcp-proxy [GLOBAL OPTIONS]**

## GLOBAL OPTIONS

#### **--dir**=*path*

The directory option is a path to store the lease backup files. The default is
*/var/tmp/nv-proxy*.

#### **--uds**
Set the unix domain socket path instead of using the default.

#### **--help**, **-h**

Print usage statement

#### **--version**, **-v**

Print the version


## HISTORY
Sep 2022, Originally compiled by Brent Baude<baude@redhat.com>
