#!/usr/bin/env bash
# Written in [Amber](https://amber-lang.com/)
# version: nightly-8-g867be7b
# Output
# True
# False
# Compiled
is_even__0_v0() {
    local n_0="${1}"
    if [ "$(( n_0 == 0 ))" != 0 ]; then
        echo "True"
        ret_is_even0_v0=1
        return 0
    fi
    is_odd__1_v0 "$(( n_0 - 1 ))"
    ret_is_even0_v0="${ret_is_odd1_v0}"
    return 0
}

is_odd__1_v0() {
    local n_1="${1}"
    if [ "$(( n_1 == 0 ))" != 0 ]; then
        echo "False"
        ret_is_odd1_v0=0
        return 0
    fi
    is_even__0_v0 "$(( n_1 - 1 ))"
    ret_is_odd1_v0="${ret_is_even0_v0}"
    return 0
}

is_even__0_v0 2
__status=$?
if [ "${__status}" != 0 ]; then
    echo "Failed 2"
fi
is_odd__1_v0 3
__status=$?
if [ "${__status}" != 0 ]; then
    echo "Failed 3"
fi
echo "Compiled"
