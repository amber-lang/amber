function has_failed__19_v0 {
    local command=$1
            eval ${command} > /dev/null 2>&1
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
:
fi;
    __AMBER_FUN_has_failed19_v0=$(echo $__AMBER_STATUS '!=' 0 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//');
    return 0
};
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
__0_name="AmberNative";
__1_target="amber";
__2_tag="0.3.1-alpha";
__3_place="/opt/amber";
has_failed__19_v0 "uname -a";
__AMBER_FUN_has_failed19_v0__7=${__AMBER_FUN_has_failed19_v0};
__AMBER_VAL_0=$(uname -a);
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
:
fi;
__4_agent=$(if [ ${__AMBER_FUN_has_failed19_v0__7} != 0 ]; then echo "unknown"; else echo "${__AMBER_VAL_0}"; fi);
echo "";

    __AMBER_VAL_1=$(uname -s);
    __AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
        echo "Failed to determine OS type.";
        echo "Please try again or use another download method.";
        exit__20_v0 1;
        __AMBER_FUN_exit20_v0__18=${__AMBER_FUN_exit20_v0};
        echo ${__AMBER_FUN_exit20_v0__18} > /dev/null 2>&1
fi;
    os_type="${__AMBER_VAL_1}";
    os=$(if [ $([ "_${os_type}" != "_Darwin" ]; echo $?) != 0 ]; then echo "macos"; else echo "linux"; fi);
    __AMBER_VAL_2=$(uname -m);
    __AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
        echo "Failed to determine architecture.";
        echo "Please try again or use another download method.";
        exit__20_v0 1;
        __AMBER_FUN_exit20_v0__28=${__AMBER_FUN_exit20_v0};
        echo ${__AMBER_FUN_exit20_v0__28} > /dev/null 2>&1
fi;
    arch_type="${__AMBER_VAL_2}";
    __AMBER_ARRAY_0=("arm64" "aarch64");
    includes__21_v0 __AMBER_ARRAY_0[@] "${arch_type}";
    __AMBER_FUN_includes21_v0__30=${__AMBER_FUN_includes21_v0};
    arch=$(if [ ${__AMBER_FUN_includes21_v0__30} != 0 ]; then echo "aarch64"; else echo "x86_64"; fi);
    url="https://github.com/Ph0enixKM/${__0_name}/releases/download/${__2_tag}/amber_${os}_${arch}";
            test -d "${__3_place}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
:
fi;
    if [ $(echo $__AMBER_STATUS '==' 0 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//') != 0 ]; then
        echo "Amber already installed";
        echo "It seems that Amber is already installed on your system. (${__3_place})";
        echo "If you want to reinstall Amber, uninstall it first.";
        echo "(Find out more at https://docs.amber-lang.com/getting_started/installation#uninstallation)";
        exit__20_v0 2;
        __AMBER_FUN_exit20_v0__45=${__AMBER_FUN_exit20_v0};
        echo ${__AMBER_FUN_exit20_v0__45} > /dev/null 2>&1
fi;
    has_failed__19_v0 "curl -V";
    __AMBER_FUN_has_failed19_v0__49=${__AMBER_FUN_has_failed19_v0};
    if [ ${__AMBER_FUN_has_failed19_v0__49} != 0 ]; then
        echo "Curl is not installed on your system.";
        echo "Please install \`curl\` and try again.";
        exit__20_v0 1;
        __AMBER_FUN_exit20_v0__52=${__AMBER_FUN_exit20_v0};
        echo ${__AMBER_FUN_exit20_v0__52} > /dev/null 2>&1
fi;
    echo "Installing Amber... 🚀";
            curl -L -o "${__1_target}" "${url}" > /dev/null 2>&1
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
            echo "Curl failed to download amber.";
            echo "Something went wrong. Please try again later.";
            exit__20_v0 1 > /dev/null 2>&1;
            __AMBER_FUN_exit20_v0__61=${__AMBER_FUN_exit20_v0};
            echo ${__AMBER_FUN_exit20_v0__61} > /dev/null 2>&1
fi;
            sudo mkdir "${__3_place}" > /dev/null 2>&1
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
            echo "Failed to create directory for amber.";
            echo "Please make sure that root user can access /opt directory.";
            exit__20_v0 1 > /dev/null 2>&1;
            __AMBER_FUN_exit20_v0__68=${__AMBER_FUN_exit20_v0};
            echo ${__AMBER_FUN_exit20_v0__68} > /dev/null 2>&1
fi;
    sudo mv "${__1_target}" "${__3_place}/${__1_target}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
        echo "Failed to move amber to the installation directory.";
        echo "Please make sure that root user can access /opt directory.";
        exit__20_v0 1;
        __AMBER_FUN_exit20_v0__74=${__AMBER_FUN_exit20_v0};
        echo ${__AMBER_FUN_exit20_v0__74} > /dev/null 2>&1
fi;
    sudo chmod +x "${__3_place}/${__1_target}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
        echo "Failed to give permissions to execute amber.";
        echo "Please make sure that root user can access /opt directory.";
        exit__20_v0 1;
        __AMBER_FUN_exit20_v0__80=${__AMBER_FUN_exit20_v0};
        echo ${__AMBER_FUN_exit20_v0__80} > /dev/null 2>&1
fi;
    sudo ln -s "${__3_place}/${__1_target}" "/usr/local/bin/${__1_target}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
        echo "Failed to create amber symbol link.";
        echo "Please make sure that root user can access /usr/local/bin directory.";
        exit__20_v0 1;
        __AMBER_FUN_exit20_v0__86=${__AMBER_FUN_exit20_v0};
        echo ${__AMBER_FUN_exit20_v0__86} > /dev/null 2>&1
fi;
            curl -G --data-urlencode "agent=${__4_agent}" --data-urlencode "name=download" "https://amber-lang.com/api/visit" > /dev/null 2>&1
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
:
fi;
    echo "Amber has been installed successfully. 🎉";
    echo "> Now you can use amber by typing \`amber\` in your terminal."