#!/usr/bin/env bats   -*- bats -*-
#
# basic netavark tests
#

load helpers

@test "basic teardown" {
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

       run_setup "$input_config"
       # Read the lease file
       run_helper cat "$TMP_TESTDIR/nv-leases"
       before=$output
       # Check that our mac address is in the lease file which
       # ensures that it was added
       run_helper jq "has(\"$random_mac\")" <<<"$before"
       assert "$output" == "true"
       # Run teardown
       run_teardown "$input_config"
       run_helper cat "$TMP_TESTDIR/nv-leases"
       # Check that the length of the lease file is now zero
       run_helper jq ". | length" <<<"$output"
       assert "$output" == 0

}
