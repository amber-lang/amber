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
__0_name="AmberNative";
__1_target="amber";
__2_tag="0.3.1-alpha";
__3_place="/opt/amber";
__4_bins_folder="/usr/local/bin";
has_failed__19_v0 "uname -a";
__AMBER_FUN_has_failed19_v0__8=${__AMBER_FUN_has_failed19_v0};
__AMBER_VAL_0=$(uname -a);
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
:
fi;
__5_agent=$(if [ ${__AMBER_FUN_has_failed19_v0__8} != 0 ]; then echo "unknown"; else echo "${__AMBER_VAL_0}"; fi);
echo "";
args=$1
    __AMBER_VAL_1=$(uname -s);
    __AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
        echo "Failed to determine OS type.";
        echo "Please try again or use another download method.";
        exit__20_v0 1;
        __AMBER_FUN_exit20_v0__19=${__AMBER_FUN_exit20_v0};
        echo ${__AMBER_FUN_exit20_v0__19} > /dev/null 2>&1
fi;
    os_type="${__AMBER_VAL_1}";
    os=$(if [ $([ "_${os_type}" != "_Darwin" ]; echo $?) != 0 ]; then echo "macos"; else echo "linux"; fi);
    __AMBER_VAL_2=$(uname -m);
    __AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
        echo "Failed to determine architecture.";
        echo "Please try again or use another download method.";
        exit__20_v0 1;
        __AMBER_FUN_exit20_v0__29=${__AMBER_FUN_exit20_v0};
        echo ${__AMBER_FUN_exit20_v0__29} > /dev/null 2>&1
fi;
    arch_type="${__AMBER_VAL_2}";
    __AMBER_ARRAY_0=("arm64" "aarch64");
    includes__21_v0 __AMBER_ARRAY_0[@] "${arch_type}";
    __AMBER_FUN_includes21_v0__31=${__AMBER_FUN_includes21_v0};
    arch=$(if [ ${__AMBER_FUN_includes21_v0__31} != 0 ]; then echo "aarch64"; else echo "x86_64"; fi);
    includes__21_v1 "${args}" "--user";
    __AMBER_FUN_includes21_v1__35=${__AMBER_FUN_includes21_v1};
    user_only_install=${__AMBER_FUN_includes21_v1__35};
    if [ ${user_only_install} != 0 ]; then
        __AMBER_VAL_3=$(echo $HOME);
        __AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
            echo "User installation requested, but unable to retrieve home directory from $HOME environment.";
            exit__20_v0 1;
            __AMBER_FUN_exit20_v0__39=${__AMBER_FUN_exit20_v0};
            echo ${__AMBER_FUN_exit20_v0__39} > /dev/null 2>&1
fi;
        home="${__AMBER_VAL_3}";
        if [ $([ "_${home}" != "_" ]; echo $?) != 0 ]; then
            echo "User installation requested, but unable to find home directory.";
            exit__20_v0 1;
            __AMBER_FUN_exit20_v0__43=${__AMBER_FUN_exit20_v0};
            echo ${__AMBER_FUN_exit20_v0__43} > /dev/null 2>&1
fi;
        __3_place="${home}/.local/lib/${arch}/amber";
        __4_bins_folder="${home}/.local/bin"
fi;
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
        __AMBER_FUN_exit20_v0__57=${__AMBER_FUN_exit20_v0};
        echo ${__AMBER_FUN_exit20_v0__57} > /dev/null 2>&1
fi;
    has_failed__19_v0 "curl -V";
    __AMBER_FUN_has_failed19_v0__61=${__AMBER_FUN_has_failed19_v0};
    if [ ${__AMBER_FUN_has_failed19_v0__61} != 0 ]; then
        echo "Curl is not installed on your system.";
        echo "Please install \`curl\` and try again.";
        exit__20_v0 1;
        __AMBER_FUN_exit20_v0__64=${__AMBER_FUN_exit20_v0};
        echo ${__AMBER_FUN_exit20_v0__64} > /dev/null 2>&1
fi;
    echo "Installing Amber... 🚀";
    sudo=$(if [ ${user_only_install} != 0 ]; then echo ""; else echo "sudo"; fi);
            ${sudo} mkdir -p "${__3_place}" > /dev/null 2>&1
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
            echo "Failed to create directory for amber.";
            if [ ${user_only_install} != 0 ]; then
                echo "Please make sure that root user can access ${__3_place} directory."
else
                echo "Please make sure that your user can access ${__3_place} directory."
fi;
            exit__20_v0 1 > /dev/null 2>&1;
            __AMBER_FUN_exit20_v0__80=${__AMBER_FUN_exit20_v0};
            echo ${__AMBER_FUN_exit20_v0__80} > /dev/null 2>&1
fi;
    if [ ${user_only_install} != 0 ]; then
                    mkdir -p "${__4_bins_folder}" > /dev/null 2>&1
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
                echo "Failed to create directory for amber bin at ${__4_bins_folder}.";
                exit__20_v0 1 > /dev/null 2>&1;
                __AMBER_FUN_exit20_v0__85=${__AMBER_FUN_exit20_v0};
                echo ${__AMBER_FUN_exit20_v0__85} > /dev/null 2>&1
fi
fi;
    url="https://github.com/Ph0enixKM/${__0_name}/releases/download/${__2_tag}/amber_${os}_${arch}";
            curl -L -o "${__1_target}" "${url}" > /dev/null 2>&1
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
            echo "Curl failed to download amber.";
            echo "Something went wrong. Please try again later.";
            exit__20_v0 1 > /dev/null 2>&1;
            __AMBER_FUN_exit20_v0__96=${__AMBER_FUN_exit20_v0};
            echo ${__AMBER_FUN_exit20_v0__96} > /dev/null 2>&1
fi;
    ${sudo} mv "${__1_target}" "${__3_place}/${__1_target}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
        echo "Failed to move amber to the installation directory.";
        echo "Please make sure that root user can access /opt directory.";
        exit__20_v0 1;
        __AMBER_FUN_exit20_v0__103=${__AMBER_FUN_exit20_v0};
        echo ${__AMBER_FUN_exit20_v0__103} > /dev/null 2>&1
fi;
    ${sudo} chmod +x "${__3_place}/${__1_target}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
        echo "Failed to give permissions to execute amber.";
        echo "Please make sure that root user can access /opt directory.";
        exit__20_v0 1;
        __AMBER_FUN_exit20_v0__109=${__AMBER_FUN_exit20_v0};
        echo ${__AMBER_FUN_exit20_v0__109} > /dev/null 2>&1
fi;
    ${sudo} ln -s "${__3_place}/${__1_target}" "${__4_bins_folder}/${__1_target}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
        echo "Failed to create amber symbol link.";
        echo "Please make sure that root user can access /usr/local/bin directory.";
        exit__20_v0 1;
        __AMBER_FUN_exit20_v0__116=${__AMBER_FUN_exit20_v0};
        echo ${__AMBER_FUN_exit20_v0__116} > /dev/null 2>&1
fi;
            curl -G --data-urlencode "agent=${__5_agent}" --data-urlencode "name=download" "https://amber-lang.com/api/visit" > /dev/null 2>&1
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
:
fi;
    echo "Amber has been installed successfully. 🎉";
    echo "> Now you can use amber by typing \`amber\` in your terminal.";
    if [ ${user_only_install} != 0 ]; then
        echo "> Since you requested a user only install with \`--user\` ensure that ~/.local/bin is in your \$PATH."
fi