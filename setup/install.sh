#!/bin/bash
# Written in [Amber](https://amber-lang.com/)
# This is the runtime dependency checker. Please do not remove these lines.
AMBER_RDC_CD=('eval' 'exit' '[[' 'test' 'sudo' 'test' 'mkdir' 'curl' 'mv' 'chmod' 'ln' 'curl')
AMBER_RDC_MD=()
for d in "${AMBER_RDC_CD[@]}"
do
    if ! command -v $d > /dev/null 2>&1; then
        AMBER_RDC_MD+=($d)
    fi
done

if (( ${#AMBER_RDC_MD[@]} != 0 )); then
    >&2 echo This program requires for these commands: \( $AMBER_RDC_MD \) to be present in \$PATH.
    exit 1
fi
unset $AMBER_RDC_CD
unset $AMBER_RDC_MD
# Dependencies are ok at this point

function has_failed__20_v0 {
    local command=$1
            eval ${command} > /dev/null 2>&1
__AS=$?
    __AF_has_failed20_v0=$(echo $__AS '!=' 0 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//');
    return 0
}
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
function get_os__29_v0 {
    __AMBER_VAL_0=$(uname -s);
    __AS=$?;
if [ $__AS != 0 ]; then
        echo "Failed to determine OS type."
        echo "Please try again or use another download method."
        exit__21_v0 1;
        __AF_exit21_v0__8=$__AF_exit21_v0;
        echo $__AF_exit21_v0__8 > /dev/null 2>&1
fi;
    local os_type="${__AMBER_VAL_0}"
    local os=$(if [ $([ "_${os_type}" != "_Darwin" ]; echo $?) != 0 ]; then echo "macos"; else echo "linux"; fi)
    __AF_get_os29_v0="${os}";
    return 0
}
function get_arch__30_v0 {
    __AMBER_VAL_1=$(uname -m);
    __AS=$?;
if [ $__AS != 0 ]; then
        echo "Failed to determine architecture."
        echo "Please try again or use another download method."
        exit__21_v0 1;
        __AF_exit21_v0__22=$__AF_exit21_v0;
        echo $__AF_exit21_v0__22 > /dev/null 2>&1
fi;
    local arch_type="${__AMBER_VAL_1}"
    __AMBER_ARRAY_0=("arm64" "aarch64");
    includes__22_v0 __AMBER_ARRAY_0[@] "${arch_type}";
    __AF_includes22_v0__25=$__AF_includes22_v0;
    local arch=$(if [ $__AF_includes22_v0__25 != 0 ]; then echo "aarch64"; else echo "x86_64"; fi)
    __AF_get_arch30_v0="${arch}";
    return 0
}
function get_home__31_v0 {
    __AMBER_VAL_2=$(echo $HOME);
    __AS=$?;
if [ $__AS != 0 ]; then
        echo "User installation requested, but unable to retrieve home directory from $HOME environment."
        exit__21_v0 1;
        __AF_exit21_v0__35=$__AF_exit21_v0;
        echo $__AF_exit21_v0__35 > /dev/null 2>&1
fi;
    local home="${__AMBER_VAL_2}"
    if [ $([ "_${home}" != "_" ]; echo $?) != 0 ]; then
        echo "User installation requested, but unable to find home directory."
        exit__21_v0 1;
        __AF_exit21_v0__39=$__AF_exit21_v0;
        echo $__AF_exit21_v0__39 > /dev/null 2>&1
fi
    __AF_get_home31_v0="${home}";
    return 0
}
function get_bins_folder__32_v0 {
    local user_only=$1
    if [ ${user_only} != 0 ]; then
        get_home__31_v0 ;
        __AF_get_home31_v0__46="${__AF_get_home31_v0}";
        __AF_get_bins_folder32_v0="${__AF_get_home31_v0__46}/.local/bin";
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
                        exit__21_v0 1 > /dev/null 2>&1;
                        __AF_exit21_v0__53=$__AF_exit21_v0;
                        echo $__AF_exit21_v0__53 > /dev/null 2>&1
fi
fi
        __AF_get_bins_folder32_v0="${bins_folder}";
        return 0
fi
}
function get_place__33_v0 {
    local user_only=$1
    if [ ${user_only} != 0 ]; then
        get_home__31_v0 ;
        __AF_get_home31_v0__62="${__AF_get_home31_v0}";
        get_arch__30_v0 ;
        __AF_get_arch30_v0__62="${__AF_get_arch30_v0}";
        __AF_get_place33_v0="${__AF_get_home31_v0__62}/.local/lib/${__AF_get_arch30_v0__62}/amber";
        return 0
else
        __AF_get_place33_v0="/opt/amber";
        return 0
fi
}
__0_name="AmberNative"
__1_target="amber"
__2_tag="0.3.1-alpha"
has_failed__20_v0 "uname -a";
__AF_has_failed20_v0__7=$__AF_has_failed20_v0;
__AMBER_VAL_3=$(uname -a);
__AS=$?;
__3_agent=$(if [ $__AF_has_failed20_v0__7 != 0 ]; then echo "unknown"; else echo "${__AMBER_VAL_3}"; fi)
echo ""
args=$1
    get_os__29_v0 ;
    __AF_get_os29_v0__14="${__AF_get_os29_v0}";
    os="${__AF_get_os29_v0__14}"
    get_arch__30_v0 ;
    __AF_get_arch30_v0__15="${__AF_get_arch30_v0}";
    arch="${__AF_get_arch30_v0__15}"
    includes__22_v1 "${args}" "--user";
    __AF_includes22_v1__17=$__AF_includes22_v1;
    user_only_install=$__AF_includes22_v1__17
    get_place__33_v0 ${user_only_install};
    __AF_get_place33_v0__18="${__AF_get_place33_v0}";
    place="${__AF_get_place33_v0__18}"
    get_bins_folder__32_v0 ${user_only_install};
    __AF_get_bins_folder32_v0__19="${__AF_get_bins_folder32_v0}";
    bins_folder="${__AF_get_bins_folder32_v0__19}"
            test -d "${place}"
__AS=$?
    if [ $(echo $__AS '==' 0 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//') != 0 ]; then
        echo "Amber already installed"
        echo "It seems that Amber is already installed on your system. (${place})"
        echo "If you want to reinstall Amber, uninstall it first."
        echo "(Find out more at https://docs.amber-lang.com/getting_started/installation#uninstallation)"
        exit__21_v0 2;
        __AF_exit21_v0__29=$__AF_exit21_v0;
        echo $__AF_exit21_v0__29 > /dev/null 2>&1
fi
    has_failed__20_v0 "curl -V";
    __AF_has_failed20_v0__33=$__AF_has_failed20_v0;
    if [ $__AF_has_failed20_v0__33 != 0 ]; then
        echo "Curl is not installed on your system."
        echo "Please install \`curl\` and try again."
        exit__21_v0 1;
        __AF_exit21_v0__36=$__AF_exit21_v0;
        echo $__AF_exit21_v0__36 > /dev/null 2>&1
fi
    echo "Installing Amber... 🚀"
    sudo=$(if [ ${user_only_install} != 0 ]; then echo ""; else echo "sudo"; fi)
            ${sudo} mkdir -p "${place}" > /dev/null 2>&1
__AS=$?;
if [ $__AS != 0 ]; then
            echo "Failed to create directory for amber."
            if [ ${user_only_install} != 0 ]; then
                echo "Please make sure that root user can access ${place} directory."
else
                echo "Please make sure that your user can access ${place} directory."
fi
            exit__21_v0 1 > /dev/null 2>&1;
            __AF_exit21_v0__52=$__AF_exit21_v0;
            echo $__AF_exit21_v0__52 > /dev/null 2>&1
fi
    if [ ${user_only_install} != 0 ]; then
                    mkdir -p "${bins_folder}" > /dev/null 2>&1
__AS=$?;
if [ $__AS != 0 ]; then
                echo "Failed to create directory for amber bin at ${bins_folder}."
                exit__21_v0 1 > /dev/null 2>&1;
                __AF_exit21_v0__57=$__AF_exit21_v0;
                echo $__AF_exit21_v0__57 > /dev/null 2>&1
fi
fi
    url="https://github.com/Ph0enixKM/${__0_name}/releases/download/${__2_tag}/amber_${os}_${arch}"
            curl -L -o "${__1_target}" "${url}" > /dev/null 2>&1
__AS=$?;
if [ $__AS != 0 ]; then
            echo "Curl failed to download amber."
            echo "Something went wrong. Please try again later."
            exit__21_v0 1 > /dev/null 2>&1;
            __AF_exit21_v0__68=$__AF_exit21_v0;
            echo $__AF_exit21_v0__68 > /dev/null 2>&1
fi
    ${sudo} mv "${__1_target}" "${place}/${__1_target}"
__AS=$?;
if [ $__AS != 0 ]; then
        echo "Failed to move amber to the installation directory."
        echo "Please make sure that root user can access /opt directory."
        exit__21_v0 1;
        __AF_exit21_v0__75=$__AF_exit21_v0;
        echo $__AF_exit21_v0__75 > /dev/null 2>&1
fi
    ${sudo} chmod +x "${place}/${__1_target}"
__AS=$?;
if [ $__AS != 0 ]; then
        echo "Failed to give permissions to execute amber."
        echo "Please make sure that root user can access /opt directory."
        exit__21_v0 1;
        __AF_exit21_v0__81=$__AF_exit21_v0;
        echo $__AF_exit21_v0__81 > /dev/null 2>&1
fi
    ${sudo} ln -s "${place}/${__1_target}" "${bins_folder}/${__1_target}"
__AS=$?;
if [ $__AS != 0 ]; then
        echo "Failed to create amber symbol link."
        echo "Please make sure that root user can access /usr/local/bin directory."
        exit__21_v0 1;
        __AF_exit21_v0__88=$__AF_exit21_v0;
        echo $__AF_exit21_v0__88 > /dev/null 2>&1
fi
            curl -G --data-urlencode "agent=${__3_agent}" --data-urlencode "name=download" "https://amber-lang.com/api/visit" > /dev/null 2>&1
__AS=$?
    echo "Amber has been installed successfully. 🎉"
    echo "> Now you can use amber by typing \`amber\` in your terminal."
    if [ ${user_only_install} != 0 ]; then
        echo "> Since you requested a user only install with \`--user\` ensure that ~/.local/bin is in your \$PATH."
fi