#!/bin/bash
#
# This is just a helper for testing dhcp macvlan and the proxy.  It
# can be used to generate a config file for the proxy-client to read
# in. i.e. this is what netavark would be sending.
#

if [ "$#" -ne 3 ]; then
    echo "usage: basic.sh <host_ifc> <netns> <netns_ifc>"
    exit 1
fi

inside_mac=$(ip netns exec ${2} cat /sys/class/net/${3}/address)


read -r -d '\0' input_config <<EOF

{
  "host_iface": "${1}",
  "container_iface": "${3}",
  "container_mac_addr": "${inside_mac}",
  "domain_name": "example.com",
  "host_name": "foobar",
  "version": 0,
  "ns_path": "/run/netns/$2"
}
  \0
EOF

 echo "$input_config" | jq

