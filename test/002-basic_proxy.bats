#!/usr/bin/env bats   -*- bats -*-
#
# basic netavark tests
#

load helpers

@test "basic proxy" {
        run_client
        assert `echo "$output" | jq -r .v4.siaddr` == $(gateway_from_subnet "$SUBNET_CIDR")
}

