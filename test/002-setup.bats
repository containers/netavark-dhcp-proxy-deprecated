#!/usr/bin/env bats   -*- bats -*-
#
# basic netavark tests
#

load helpers

@test "basic setup" {
      read -r -d '\0' input_config <<EOF
{
  "iface": "veth0",
  "mac_addr": "3c:e1:a1:c1:7a:3f",
  "domain_name": "example.com",
  "host_name": "foobar",
  "version": 0
}
  \0
EOF

        run_setup "$input_config"
        # Check that gateway provided is the first IP in the subnet
        assert `echo "$output" | jq -r .v4.siaddr` == $(gateway_from_subnet "$SUBNET_CIDR")
}

@test "empty interface should fail" {
      read -r -d '\0' input_config <<EOF
{
  "iface": "",
  "mac_addr": "3c:e1:a1:c1:7a:3f",
  "domain_name": "example.com",
  "host_name": "foobar",
  "version": 0
}
  \0
EOF
        # Not providing an interface in the config should result
        # in an error and a return code of 156
        expected_rc=156 run_setup "$input_config"
}

@test "empty mac address should fail" {
      read -r -d '\0' input_config <<EOF
{
  "iface": "veth0",
  "mac_addr": "",
  "domain_name": "example.com",
  "host_name": "foobar",
  "version": 0
}
  \0
EOF
        # Not mac address should result in an error
        # and return code of 156
        expected_rc=156 run_setup "$input_config"
}

@test "invalid interface should fail" {
      read -r -d '\0' input_config <<EOF
{
  "iface": "veth0",
  "mac_addr": "",
  "domain_name": "example.com",
  "host_name": "foobar",
  "version": 0
}
  \0
EOF
        # A non-existent interface should result in an
        # error and return code of 156
        expected_rc=156 run_setup "$input_config"
}

@test "invalid mac address should fail" {
      read -r -d '\0' input_config <<EOF
{
  "iface": "veth0",
  "mac_addr": "123",
  "domain_name": "example.com",
  "host_name": "foobar",
  "version": 0
}
  \0
EOF

        # An invalid mac address should result in an
        # error and a return code of 156
        expected_rc=156 run_setup "$input_config"
}
