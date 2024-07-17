#!/usr/bin/env bash
# Written in [Amber](https://amber-lang.com/)

function exit__23_v0 {
    local code=$1
            exit "${code}"
__AS=$?
}
function includes__24_v0 {
    local arr=("${!1}")
    local value=$2
    for v in "${arr[@]}"
do
        if [ $([ "_${v}" != "_${value}" ]; echo $?) != 0 ]; then
            __AF_includes24_v0=1;
            return 0
fi
done
    __AF_includes24_v0=0;
    return 0
}
function get_arch__31_v0 {
    __AMBER_VAL_0=$(uname -m);
    __AS=$?;
if [ $__AS != 0 ]; then
        echo "Failed to determine architecture."
        echo "Please try again or use another download method."
        exit__23_v0 1;
        __AF_exit23_v0__30=$__AF_exit23_v0;
        echo $__AF_exit23_v0__30 > /dev/null 2>&1
fi;
    local arch_type="${__AMBER_VAL_0}"
    __AMBER_ARRAY_0=("arm64" "aarch64");
    includes__24_v0 __AMBER_ARRAY_0[@] "${arch_type}";
    __AF_includes24_v0__33=$__AF_includes24_v0;
    local arch=$(if [ $__AF_includes24_v0__33 != 0 ]; then echo "aarch64"; else echo "x86_64"; fi)
    __AF_get_arch31_v0="${arch}";
    return 0
}
function get_home__32_v0 {
    __AMBER_VAL_1=$(echo $HOME);
    __AS=$?;
if [ $__AS != 0 ]; then
        echo "User installation requested, but unable to retrieve home directory from $HOME environment."
        exit__23_v0 1;
        __AF_exit23_v0__43=$__AF_exit23_v0;
        echo $__AF_exit23_v0__43 > /dev/null 2>&1
fi;
    local home="${__AMBER_VAL_1}"
    if [ $([ "_${home}" != "_" ]; echo $?) != 0 ]; then
        echo "User installation requested, but unable to find home directory."
        exit__23_v0 1;
        __AF_exit23_v0__47=$__AF_exit23_v0;
        echo $__AF_exit23_v0__47 > /dev/null 2>&1
fi
    __AF_get_home32_v0="${home}";
    return 0
}
function get_bins_folder__33_v0 {
    local user_only=$1
    if [ ${user_only} != 0 ]; then
        get_home__32_v0 ;
        __AF_get_home32_v0__54="${__AF_get_home32_v0}";
        __AF_get_bins_folder33_v0="${__AF_get_home32_v0__54}/.local/bin";
        return 0
else
        local bins_folder="/usr/local/bin"
                    test -d "${bins_folder}"
__AS=$?;
if [ $__AS != 0 ]; then
                                    sudo mkdir -p "${bins_folder}" > /dev/null 2>&1
__AS=$?;
if [ $__AS != 0 ]; then
                        echo "Failed to create ${bins_folder} directory."
                        exit__23_v0 1 > /dev/null 2>&1;
                        __AF_exit23_v0__61=$__AF_exit23_v0;
                        echo $__AF_exit23_v0__61 > /dev/null 2>&1
fi
fi
        __AF_get_bins_folder33_v0="${bins_folder}";
        return 0
fi
}
function get_place__34_v0 {
    local user_only=$1
    if [ ${user_only} != 0 ]; then
        get_home__32_v0 ;
        __AF_get_home32_v0__70="${__AF_get_home32_v0}";
        get_arch__31_v0 ;
        __AF_get_arch31_v0__70="${__AF_get_arch31_v0}";
        __AF_get_place34_v0="${__AF_get_home32_v0__70}/.local/lib/${__AF_get_arch31_v0__70}/amber";
        return 0
else
        __AF_get_place34_v0="/opt/amber";
        return 0
fi
}
echo ""
args=("$@")
    get_arch__31_v0 ;
    __AF_get_arch31_v0__8="${__AF_get_arch31_v0}";
    arch="${__AF_get_arch31_v0__8}"
    includes__24_v0 args[@] "--user";
    __AF_includes24_v0__10=$__AF_includes24_v0;
    user_only_install=$__AF_includes24_v0__10
    get_place__34_v0 ${user_only_install};
    __AF_get_place34_v0__11="${__AF_get_place34_v0}";
    place="${__AF_get_place34_v0__11}"
    get_bins_folder__33_v0 ${user_only_install};
    __AF_get_bins_folder33_v0__12="${__AF_get_bins_folder33_v0}";
    bins_folder="${__AF_get_bins_folder33_v0__12}"
            test -d "${place}" > /dev/null
__AS=$?
    if [ $(echo $__AS '==' 0 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//') != 0 ]; then
        sudo=$(if [ ${user_only_install} != 0 ]; then echo ""; else echo "sudo"; fi)
        ${sudo} rm -rf "${place}"
__AS=$?;
if [ $__AS != 0 ]; then
            echo "Failed to remove Amber from ${place}"
            echo "Make sure root has the correct permissions to access this directory"
            exit__23_v0 1;
            __AF_exit23_v0__21=$__AF_exit23_v0;
            echo $__AF_exit23_v0__21 > /dev/null 2>&1
fi
        ${sudo} rm "${bins_folder}/amber"
__AS=$?;
if [ $__AS != 0 ]; then
            echo "Failed to remove Amber symlink from ${bins_folder}"
            echo "Make sure root has the correct permissions to access this directory"
            exit__23_v0 1;
            __AF_exit23_v0__26=$__AF_exit23_v0;
            echo $__AF_exit23_v0__26 > /dev/null 2>&1
fi
        echo "Uninstalled Amber successfully 🎉"
else
        echo "Amber is not installed"
fi