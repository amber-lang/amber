#!/usr/bin/env bash
# Written in [Amber](https://amber-lang.com/)
# version: b6d74a1
# Output
# 
you_do_not_have_this >/dev/null 2>&1
__status=$?
if [ "${__status}" = 0 ]; then
    echo "This should not execute"'!'""
fi
