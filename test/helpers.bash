# -*- bash -*-

# Netavark binary to run
NETAVARK=${NETAVARK:-./bin/netavark}

TESTSDIR=${TESTSDIR:-$(dirname ${BASH_SOURCE})}

# export RUST_BACKTRACE so that we get a helpful stack trace
export RUST_BACKTRACE=full
