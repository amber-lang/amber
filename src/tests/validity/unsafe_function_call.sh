#!/usr/bin/env bash
# Written in [Amber](https://amber-lang.com/)
# version: 48516ff
# Output
# 6, 0
safe_division__0_v0() {
    local a=$1
    local b=$2
    if [ "$(( ${b} == 0 ))" != 0 ]; then
        __ret_safe_division0_v0=''
        return 1
    fi
    __ret_safe_division0_v0="$(( ${a} / ${b} ))"
    return 0
}

safe_division__0_v0 24 4
__0_result="${__ret_safe_division0_v0}"
echo "${__0_result}, ${__status}"
