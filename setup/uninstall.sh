function exit__21_v0 {
    local code=$1
            exit "${code}"
__AS=$?
}
function includes__22_v0 {
    local arr=("${!1}")
    local value=$2
            [[ "${arr[@]}" =~ "${value}" ]]
__AS=$?
    __AF_includes22_v0=$(echo $__AS '==' 0 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//');
    return 0
}
function includes__22_v1 {
    local arr=$1
    local value=$2
            [[ "${arr[@]}" =~ "${value}" ]]
__AS=$?
    __AF_includes22_v1=$(echo $__AS '==' 0 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//');
    return 0
}
function get_arch__28_v0 {
    __AMBER_VAL_0=$(uname -m);
    __AS=$?;
if [ $__AS != 0 ]; then
        echo "Failed to determine architecture."
        echo "Please try again or use another download method."
        exit__21_v0 1;
        __AF_exit21_v0__22=$__AF_exit21_v0;
        echo $__AF_exit21_v0__22 > /dev/null 2>&1
fi;
    local arch_type="${__AMBER_VAL_0}"
    __AMBER_ARRAY_0=("arm64" "aarch64");
    includes__22_v0 __AMBER_ARRAY_0[@] "${arch_type}";
    __AF_includes22_v0__25=$__AF_includes22_v0;
    local arch=$(if [ $__AF_includes22_v0__25 != 0 ]; then echo "aarch64"; else echo "x86_64"; fi)
    __AF_get_arch28_v0="${arch}";
    return 0
}
function get_home__29_v0 {
    __AMBER_VAL_1=$(echo $HOME);
    __AS=$?;
if [ $__AS != 0 ]; then
        echo "User installation requested, but unable to retrieve home directory from $HOME environment."
        exit__21_v0 1;
        __AF_exit21_v0__35=$__AF_exit21_v0;
        echo $__AF_exit21_v0__35 > /dev/null 2>&1
fi;
    local home="${__AMBER_VAL_1}"
    if [ $([ "_${home}" != "_" ]; echo $?) != 0 ]; then
        echo "User installation requested, but unable to find home directory."
        exit__21_v0 1;
        __AF_exit21_v0__39=$__AF_exit21_v0;
        echo $__AF_exit21_v0__39 > /dev/null 2>&1
fi
    __AF_get_home29_v0="${home}";
    return 0
}
function get_bins_folder__30_v0 {
    local user_only=$1
    if [ ${user_only} != 0 ]; then
        get_home__29_v0 ;
        __AF_get_home29_v0__46=$__AF_get_home29_v0;
        __AF_get_bins_folder30_v0="$__AF_get_home29_v0__46/.local/bin";
        return 0
else
        __AF_get_bins_folder30_v0="/usr/local/bin";
        return 0
fi
}
function get_place__31_v0 {
    local user_only=$1
    if [ ${user_only} != 0 ]; then
        get_home__29_v0 ;
        __AF_get_home29_v0__54=$__AF_get_home29_v0;
        get_arch__28_v0 ;
        __AF_get_arch28_v0__54=$__AF_get_arch28_v0;
        __AF_get_place31_v0="$__AF_get_home29_v0__54/.local/lib/$__AF_get_arch28_v0__54/amber";
        return 0
else
        __AF_get_place31_v0="/opt/amber";
        return 0
fi
}
echo ""
args=$1
    get_arch__28_v0 ;
    __AF_get_arch28_v0__8=$__AF_get_arch28_v0;
    arch=$__AF_get_arch28_v0__8
    includes__22_v1 "${args}" "--user";
    __AF_includes22_v1__10=$__AF_includes22_v1;
    user_only_install=$__AF_includes22_v1__10
    get_place__31_v0 ${user_only_install};
    __AF_get_place31_v0__11=$__AF_get_place31_v0;
    place=$__AF_get_place31_v0__11
    get_bins_folder__30_v0 ${user_only_install};
    __AF_get_bins_folder30_v0__12=$__AF_get_bins_folder30_v0;
    bins_folder=$__AF_get_bins_folder30_v0__12
            test -d "${place}" > /dev/null
__AS=$?
    if [ $(echo $__AS '==' 0 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//') != 0 ]; then
        sudo=$(if [ ${user_only_install} != 0 ]; then echo ""; else echo "sudo"; fi)
        ${sudo} rm -rf "${place}"
__AS=$?;
if [ $__AS != 0 ]; then
            echo "Failed to remove Amber from ${place}"
            echo "Make sure root has the correct permissions to access this directory"
            exit__21_v0 1;
            __AF_exit21_v0__21=$__AF_exit21_v0;
            echo $__AF_exit21_v0__21 > /dev/null 2>&1
fi
        ${sudo} rm "${bins_folder}/amber"
__AS=$?;
if [ $__AS != 0 ]; then
            echo "Failed to remove Amber symlink from ${bins_folder}"
            echo "Make sure root has the correct permissions to access this directory"
            exit__21_v0 1;
            __AF_exit21_v0__26=$__AF_exit21_v0;
            echo $__AF_exit21_v0__26 > /dev/null 2>&1
fi
        echo "Uninstalled Amber successfully 🎉"
else
        echo "Amber is not installed"
fi