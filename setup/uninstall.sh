function exit__20_v0 {
    local code=$1
            exit "${code}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
:
fi
};
function includes__21_v0 {
    local arr=("${!1}")
    local value=$2
            [[ "${arr[@]}" =~ "${value}" ]]
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
:
fi;
    __AMBER_FUN_includes21_v0=$(echo $__AMBER_STATUS '==' 0 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//');
    return 0
}
function includes__21_v1 {
    local arr=$1
    local value=$2
            [[ "${arr[@]}" =~ "${value}" ]]
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
:
fi;
    __AMBER_FUN_includes21_v1=$(echo $__AMBER_STATUS '==' 0 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//');
    return 0
}
function get_arch__27_v0 {
    __AMBER_VAL_0=$(uname -m);
    __AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
        echo "Failed to determine architecture.";
        echo "Please try again or use another download method.";
        exit__20_v0 1;
        __AMBER_FUN_exit20_v0__22=${__AMBER_FUN_exit20_v0};
        echo ${__AMBER_FUN_exit20_v0__22} > /dev/null 2>&1
fi;
    local arch_type="${__AMBER_VAL_0}";
    __AMBER_ARRAY_0=("arm64" "aarch64");
    includes__21_v0 __AMBER_ARRAY_0[@] "${arch_type}";
    __AMBER_FUN_includes21_v0__25=${__AMBER_FUN_includes21_v0};
    local arch=$(if [ ${__AMBER_FUN_includes21_v0__25} != 0 ]; then echo "aarch64"; else echo "x86_64"; fi);
    __AMBER_FUN_get_arch27_v0="${arch}";
    return 0
};
function get_home__28_v0 {
    __AMBER_VAL_1=$(echo $HOME);
    __AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
        echo "User installation requested, but unable to retrieve home directory from $HOME environment.";
        exit__20_v0 1;
        __AMBER_FUN_exit20_v0__35=${__AMBER_FUN_exit20_v0};
        echo ${__AMBER_FUN_exit20_v0__35} > /dev/null 2>&1
fi;
    local home="${__AMBER_VAL_1}";
    if [ $([ "_${home}" != "_" ]; echo $?) != 0 ]; then
        echo "User installation requested, but unable to find home directory.";
        exit__20_v0 1;
        __AMBER_FUN_exit20_v0__39=${__AMBER_FUN_exit20_v0};
        echo ${__AMBER_FUN_exit20_v0__39} > /dev/null 2>&1
fi;
    __AMBER_FUN_get_home28_v0="${home}";
    return 0
};
function get_bins_folder__29_v0 {
    local user_only=$1
    if [ ${user_only} != 0 ]; then
        get_home__28_v0 ;
        __AMBER_FUN_get_home28_v0__46=${__AMBER_FUN_get_home28_v0};
        __AMBER_FUN_get_bins_folder29_v0="${__AMBER_FUN_get_home28_v0__46}/.local/bin";
        return 0
else
        __AMBER_FUN_get_bins_folder29_v0="/usr/local/bin";
        return 0
fi
};
function get_place__30_v0 {
    local user_only=$1
    if [ ${user_only} != 0 ]; then
        get_home__28_v0 ;
        __AMBER_FUN_get_home28_v0__54=${__AMBER_FUN_get_home28_v0};
        get_arch__27_v0 ;
        __AMBER_FUN_get_arch27_v0__54=${__AMBER_FUN_get_arch27_v0};
        __AMBER_FUN_get_place30_v0="${__AMBER_FUN_get_home28_v0__54}/.local/lib/${__AMBER_FUN_get_arch27_v0__54}/amber";
        return 0
else
        __AMBER_FUN_get_place30_v0="/opt/amber";
        return 0
fi
}
echo "";
args=$1
    get_arch__27_v0 ;
    __AMBER_FUN_get_arch27_v0__8=${__AMBER_FUN_get_arch27_v0};
    arch=${__AMBER_FUN_get_arch27_v0__8};
    includes__21_v1 "${args}" "--user";
    __AMBER_FUN_includes21_v1__10=${__AMBER_FUN_includes21_v1};
    user_only_install=${__AMBER_FUN_includes21_v1__10};
    get_place__30_v0 ${user_only_install};
    __AMBER_FUN_get_place30_v0__11=${__AMBER_FUN_get_place30_v0};
    place=${__AMBER_FUN_get_place30_v0__11};
    get_bins_folder__29_v0 ${user_only_install};
    __AMBER_FUN_get_bins_folder29_v0__12=${__AMBER_FUN_get_bins_folder29_v0};
    bins_folder=${__AMBER_FUN_get_bins_folder29_v0__12};
            test -d "${place}" > /dev/null
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
:
fi;
    if [ $(echo $__AMBER_STATUS '==' 0 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//') != 0 ]; then
        sudo=$(if [ ${user_only_install} != 0 ]; then echo ""; else echo "sudo"; fi);
        ${sudo} rm -rf "${place}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
            echo "Failed to remove Amber from ${place}";
            echo "Make sure root has the correct permissions to access this directory";
            exit__20_v0 1;
            __AMBER_FUN_exit20_v0__21=${__AMBER_FUN_exit20_v0};
            echo ${__AMBER_FUN_exit20_v0__21} > /dev/null 2>&1
fi;
        ${sudo} rm "${bins_folder}/amber"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
            echo "Failed to remove Amber symlink from ${bins_folder}";
            echo "Make sure root has the correct permissions to access this directory";
            exit__20_v0 1;
            __AMBER_FUN_exit20_v0__26=${__AMBER_FUN_exit20_v0};
            echo ${__AMBER_FUN_exit20_v0__26} > /dev/null 2>&1
fi;
        echo "Uninstalled Amber successfully 🎉"
else
        echo "Amber is not installed"
fi