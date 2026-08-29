#!/usr/bin/env bash
# Written in [Amber](https://amber-lang.com/)
# version: 0.6.0-alpha-47-g887330c

# trim(text: Text)
trim__0_v0() {
    local text_1="${1}"
    ret_trim0_v0="$(echo "${text_1}" | xargs)"
    : "${ret_trim0_v0}"
    return 0
}

trim__0_v0 "long text"
