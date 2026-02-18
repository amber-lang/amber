#!/usr/bin/env bash
# Written in [Amber](https://amber-lang.com/)
# version: 6f251cf
# Output
# First lock successful
__lock_file_1=/tmp/${0##*/}.lock
if [ -f "${__lock_file_1}" ]; then
    echo "Script is already running"
    exit 1
fi
touch "${__lock_file_1}"
trap 'rm -f "/tmp/${0##*/}.lock"' EXIT

echo "First lock successful"
__lock_file_2=/tmp/${0##*/}.lock
if [ -f "${__lock_file_2}" ]; then
    echo "Script is already running"
    exit 1
fi
touch "${__lock_file_2}"
trap 'rm -f "/tmp/${0##*/}.lock"' EXIT

echo "Should not print"
