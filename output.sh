#!/usr/bin/env bash
# Written in [Amber](https://amber-lang.com/)
# version: 0.5.1-alpha-3-gb1d251b
    # Output
    # Hello World
    echo "Hello World"
    non-existent command >/dev/null 2>&1
    # Test for expression
    command_0="$(non-existent command >/dev/null 2>&1)"
    # Test for single statement
    non-existent command >/dev/null 2>&1
