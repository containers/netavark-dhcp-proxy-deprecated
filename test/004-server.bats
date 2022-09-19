#!/usr/bin/env bats   -*- bats -*-
#
# basic netavark tests
#

load helpers
@test "SIGINT Clean up" {
  random_mac=$(generate_mac)
      read -r -d '\0' input_config <<EOF
{
  "iface": "veth0",
  "mac_addr": "${random_mac}",
  "domain_name": "example.com",
  "host_name": "foobar",
  "version": 0
}
  \0
EOF

# Make sure that nv-uds socket does not exist after SIGINT
run_helper kill -s SIGINT "$PROXY_PID"
expected_rc=2 run_helper ls -l "$TMP_TESTDIR/socket"

}

@test "SIGTERM Clean up" {
random_mac=$(generate_mac)
read -r -d '\0' input_config <<EOF
{
    "iface": "veth0",
    "mac_addr": "${random_mac}",
    "domain_name": "example.com",
    "host_name": "foobar",
    "version": 0
}
  \0
EOF

# Make sure that nv-uds socket does not exist after SIGINT
run_helper kill -s SIGTERM "$PROXY_PID"
expected_rc=2 run_helper ls -l "$TMP_TESTDIR/socket"
}
