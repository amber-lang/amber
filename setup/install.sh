function has_failed__18_v0 {
    local command=$1
            eval ${command} > /dev/null 2>&1
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
:
fi;
    __AMBER_FUN_has_failed18_v0=$(echo $__AMBER_STATUS '!=' 0 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//');
    return 0
};
function exit__19_v0 {
    local code=$1
            exit "${code}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
:
fi
};
function includes__20_v0 {
    local arr=("${!1}")
    local value=$2
            [[ "${arr[@]}" =~ "${value}" ]]
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
:
fi;
    __AMBER_FUN_includes20_v0=$(echo $__AMBER_STATUS '==' 0 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//');
    return 0
}
__0_name="AmberNative";
__1_target="amber";
__2_tag="0.3.1-alpha";
__3_place="/opt/amber";

    __AMBER_VAL_0=$(uname -s);
    __AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
        echo "Failed to determine OS type.";
        echo "Please try again or use another download method.";
        exit__19_v0 1;
        __AMBER_FUN_exit19_v0__13=${__AMBER_FUN_exit19_v0};
        echo ${__AMBER_FUN_exit19_v0__13} > /dev/null 2>&1
fi;
    os_type="${__AMBER_VAL_0}";
    os=$(if [ $([ "_${os_type}" != "_Darwin" ]; echo $?) != 0 ]; then echo "macos"; else echo "linux"; fi);
    __AMBER_VAL_1=$(uname -m);
    __AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
        echo "Failed to determine architecture.";
        echo "Please try again or use another download method.";
        exit__19_v0 1;
        __AMBER_FUN_exit19_v0__23=${__AMBER_FUN_exit19_v0};
        echo ${__AMBER_FUN_exit19_v0__23} > /dev/null 2>&1
fi;
    arch_type="${__AMBER_VAL_1}";
    __AMBER_ARRAY_0=("arm64" "aarch64");
    includes__20_v0 __AMBER_ARRAY_0[@] "${arch_type}";
    __AMBER_FUN_includes20_v0__25=${__AMBER_FUN_includes20_v0};
    arch=$(if [ ${__AMBER_FUN_includes20_v0__25} != 0 ]; then echo "aarch64"; else echo "x86_64"; fi);
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
        echo "(Find out more at https://amber.marbl.cc)";
        exit__19_v0 2;
        __AMBER_FUN_exit19_v0__40=${__AMBER_FUN_exit19_v0};
        echo ${__AMBER_FUN_exit19_v0__40} > /dev/null 2>&1
fi;
    echo "Installing Amber";
    has_failed__18_v0 "ruby -v";
    __AMBER_FUN_has_failed18_v0__46=${__AMBER_FUN_has_failed18_v0};
    has_failed__18_v0 "curl -v";
    __AMBER_FUN_has_failed18_v0__55=${__AMBER_FUN_has_failed18_v0};
    has_failed__18_v0 "wget -V";
    __AMBER_FUN_has_failed18_v0__63=${__AMBER_FUN_has_failed18_v0};
    if [ $(echo  '!' ${__AMBER_FUN_has_failed18_v0__46} | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//') != 0 ]; then
        code="require \"open-uri\"; open(\"${__1_target}\", \"wb\") do |file|; file << open(\"${url}\").read; end";
        echo "Using ruby as a download method...";
        sudo ruby -e "${code}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
            echo "Ruby failed to download amber.";
            echo "Something went wrong. Please try again later.";
            exit__19_v0 1;
            __AMBER_FUN_exit19_v0__52=${__AMBER_FUN_exit19_v0};
            echo ${__AMBER_FUN_exit19_v0__52} > /dev/null 2>&1
fi
elif [ $(echo  '!' ${__AMBER_FUN_has_failed18_v0__55} | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//') != 0 ]; then
        echo "Using curl as a download method...";
        curl -o "${__1_target}" "${url}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
            echo "Curl failed to download amber.";
            echo "Something went wrong. Please try again later.";
            exit__19_v0 1;
            __AMBER_FUN_exit19_v0__60=${__AMBER_FUN_exit19_v0};
            echo ${__AMBER_FUN_exit19_v0__60} > /dev/null 2>&1
fi
elif [ $(echo  '!' ${__AMBER_FUN_has_failed18_v0__63} | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//') != 0 ]; then
        echo "Using wget as a download method...";
        wget -O "${__1_target}" "${url}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
            echo "Wget failed to download amber.";
            echo "Something went wrong. Please try again later.";
            exit__19_v0 1;
            __AMBER_FUN_exit19_v0__68=${__AMBER_FUN_exit19_v0};
            echo ${__AMBER_FUN_exit19_v0__68} > /dev/null 2>&1
fi
else
        echo "Neither ruby, curl or wget are installed on your system.";
        echo "Please install one of them and try again.";
        exit__19_v0 1;
        __AMBER_FUN_exit19_v0__74=${__AMBER_FUN_exit19_v0};
        echo ${__AMBER_FUN_exit19_v0__74} > /dev/null 2>&1
fi;
    sudo mkdir "${__3_place}" > /dev/null
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
        echo "Failed to create directory for amber.";
        echo "Please make sure that root user can access /opt directory.";
        exit__19_v0 1;
        __AMBER_FUN_exit19_v0__82=${__AMBER_FUN_exit19_v0};
        echo ${__AMBER_FUN_exit19_v0__82} > /dev/null 2>&1
fi;
    sudo mv "${__1_target}" "${__3_place}/${__1_target}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
        echo "Failed to move amber to the installation directory.";
        echo "Please make sure that root user can access /opt directory.";
        exit__19_v0 1;
        __AMBER_FUN_exit19_v0__88=${__AMBER_FUN_exit19_v0};
        echo ${__AMBER_FUN_exit19_v0__88} > /dev/null 2>&1
fi;
    sudo chmod +x "${__3_place}/${__1_target}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
        echo "Failed to give permissions to execute amber.";
        echo "Please make sure that root user can access /opt directory.";
        exit__19_v0 1;
        __AMBER_FUN_exit19_v0__94=${__AMBER_FUN_exit19_v0};
        echo ${__AMBER_FUN_exit19_v0__94} > /dev/null 2>&1
fi;
    sudo ln -s "${__3_place}/${__1_target}" "/usr/local/bin/${__1_target}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
        echo "Failed to create amber symbol link.";
        echo "Please make sure that root user can access /usr/local/bin directory.";
        exit__19_v0 1;
        __AMBER_FUN_exit19_v0__100=${__AMBER_FUN_exit19_v0};
        echo ${__AMBER_FUN_exit19_v0__100} > /dev/null 2>&1
fi;
    echo "Amber has been installed successfully. 🎉"