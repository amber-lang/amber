#!/usr/bin/env bash
# Written in [Amber](https://amber-lang.com/)
# version: 0.3.4-alpha
# date: 2024-07-22 14:15:40


function shell_escape__62_v0 {
    local text=$1
    local escape_space=$2
    __AMBER_VAL_0=$(echo $text|sed 's/\\/\\\\/g');
    __AS=$?;
    text="${__AMBER_VAL_0}"
    echo "${text}"
    __AMBER_VAL_1=$(echo $text|sed "s/%/%%/g");
    __AS=$?;
    text="${__AMBER_VAL_1}"
    echo "${text}"
    __AF_shell_escape62_v0="${text}";
    return 0
}
function text_shell__63_v0 {
    local message=$1
    local color1=$2
    local style=$3
    local color2=$4
    shell_escape__62_v0 "${message}" 0;
    __AF_shell_escape62_v0__95_82="${__AF_shell_escape62_v0}";
    __AMBER_VAL_2=$(printf "\e[${style};${color1};${color2}m${__AF_shell_escape62_v0__95_82}\e[0m");
    __AS=$?;
    __AF_text_shell63_v0="${__AMBER_VAL_2}";
    return 0
}

    text_shell__63_v0 "Hello %T \v Amber"'!'"" 31 1 42;
    __AF_text_shell63_v0__3_10="${__AF_text_shell63_v0}";
    echo "${__AF_text_shell63_v0__3_10}"