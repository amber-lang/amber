#!/usr/bin/env bash
# Written in [Amber](https://amber-lang.com/)

function has_failed__22_v0 {
    local command=$1
            eval ${command} > /dev/null 2>&1
__AS=$?
    __AF_has_failed22_v0=$(echo $__AS '!=' 0 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//');
    return 0
}
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
function get_os__31_v0 {
    __AMBER_VAL_0=$(uname -s);
    __AS=$?;
if [ $__AS != 0 ]; then
        echo "Failed to determine OS type."
        echo "Please try again or use another download method."
        exit__23_v0 1;
        __AF_exit23_v0__8=$__AF_exit23_v0;
        echo $__AF_exit23_v0__8 > /dev/null 2>&1
fi;
    local os_type="${__AMBER_VAL_0}"
    local os=$(if [ $([ "_${os_type}" != "_Darwin" ]; echo $?) != 0 ]; then echo "apple-darwin"; else echo "unknown-linux-gnu"; fi)
    __AF_get_os31_v0="${os}";
    return 0
}
function get_arch__32_v0 {
    __AMBER_VAL_1=$(uname -m);
    __AS=$?;
if [ $__AS != 0 ]; then
        echo "Failed to determine architecture."
        echo "Please try again or use another download method."
        exit__23_v0 1;
        __AF_exit23_v0__22=$__AF_exit23_v0;
        echo $__AF_exit23_v0__22 > /dev/null 2>&1
fi;
    local arch_type="${__AMBER_VAL_1}"
    __AMBER_ARRAY_0=("arm64" "aarch64");
    includes__24_v0 __AMBER_ARRAY_0[@] "${arch_type}";
    __AF_includes24_v0__25=$__AF_includes24_v0;
    local arch=$(if [ $__AF_includes24_v0__25 != 0 ]; then echo "aarch64"; else echo "x86_64"; fi)
    __AF_get_arch32_v0="${arch}";
    return 0
}
function get_home__33_v0 {
    __AMBER_VAL_2=$(echo $HOME);
    __AS=$?;
if [ $__AS != 0 ]; then
        echo "User installation requested, but unable to retrieve home directory from $HOME environment."
        exit__23_v0 1;
        __AF_exit23_v0__35=$__AF_exit23_v0;
        echo $__AF_exit23_v0__35 > /dev/null 2>&1
fi;
    local home="${__AMBER_VAL_2}"
    if [ $([ "_${home}" != "_" ]; echo $?) != 0 ]; then
        echo "User installation requested, but unable to find home directory."
        exit__23_v0 1;
        __AF_exit23_v0__39=$__AF_exit23_v0;
        echo $__AF_exit23_v0__39 > /dev/null 2>&1
fi
    __AF_get_home33_v0="${home}";
    return 0
}
function get_bins_folder__34_v0 {
    local user_only=$1
    if [ ${user_only} != 0 ]; then
        get_home__33_v0 ;
        __AF_get_home33_v0__46="${__AF_get_home33_v0}";
        __AF_get_bins_folder34_v0="${__AF_get_home33_v0__46}/.local/bin";
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
                        __AF_exit23_v0__53=$__AF_exit23_v0;
                        echo $__AF_exit23_v0__53 > /dev/null 2>&1
fi
fi
        __AF_get_bins_folder34_v0="${bins_folder}";
        return 0
fi
}
function get_place__35_v0 {
    local user_only=$1
    if [ ${user_only} != 0 ]; then
        get_home__33_v0 ;
        __AF_get_home33_v0__62="${__AF_get_home33_v0}";
        get_arch__32_v0 ;
        __AF_get_arch32_v0__62="${__AF_get_arch32_v0}";
        __AF_get_place35_v0="${__AF_get_home33_v0__62}/.local/lib/${__AF_get_arch32_v0__62}/amber";
        return 0
else
        __AF_get_place35_v0="/opt/amber";
        return 0
fi
}
__0_name="AmberNative"
__1_target="amber"
__2_archive="amber.tar.xz"
has_failed__22_v0 "uname -a";
__AF_has_failed22_v0__7=$__AF_has_failed22_v0;
__AMBER_VAL_3=$(uname -a);
__AS=$?;
__3_agent=$(if [ $__AF_has_failed22_v0__7 != 0 ]; then echo "unknown"; else echo "${__AMBER_VAL_3}"; fi)
echo ""
function get_latest_release_tag__40_v0 {
    local tag_url="https://api.github.com/repos/Ph0enixKM/${__0_name}/releases/latest"
    __AMBER_VAL_4=$(curl -L "${tag_url}" 2>/dev/null);
    __AS=$?;
if [ $__AS != 0 ]; then
__AF_get_latest_release_tag40_v0=''
return $__AS
fi;
    local tag_json="${__AMBER_VAL_4}"
    __AMBER_VAL_5=$(echo "$tag_json"         | grep -Eo "tag_name\"[^\"]*\"([^\"]+)\""         | grep -Eo "\"[^\"]+\"$"         | grep -Eo "[^\"\s]+");
    __AS=$?;
if [ $__AS != 0 ]; then
__AF_get_latest_release_tag40_v0=''
return $__AS
fi;
    local tag="${__AMBER_VAL_5}"
    __AF_get_latest_release_tag40_v0="${tag}";
    return 0
}
args=("$@")
    get_os__31_v0 ;
    __AF_get_os31_v0__25="${__AF_get_os31_v0}";
    os="${__AF_get_os31_v0__25}"
    get_arch__32_v0 ;
    __AF_get_arch32_v0__26="${__AF_get_arch32_v0}";
    arch="${__AF_get_arch32_v0__26}"
    includes__24_v0 args[@] "--user";
    __AF_includes24_v0__28=$__AF_includes24_v0;
    user_only_install=$__AF_includes24_v0__28
    get_place__35_v0 ${user_only_install};
    __AF_get_place35_v0__29="${__AF_get_place35_v0}";
    place="${__AF_get_place35_v0__29}"
    get_bins_folder__34_v0 ${user_only_install};
    __AF_get_bins_folder34_v0__30="${__AF_get_bins_folder34_v0}";
    bins_folder="${__AF_get_bins_folder34_v0__30}"
            test -d "${place}"
__AS=$?
    if [ $(echo $__AS '==' 0 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//') != 0 ]; then
        echo "Amber already installed"
        echo "It seems that Amber is already installed on your system. (${place})"
        echo "If you want to reinstall Amber, uninstall it first."
        echo "(Find out more at https://docs.amber-lang.com/getting_started/installation#uninstallation)"
        exit__23_v0 2;
        __AF_exit23_v0__40=$__AF_exit23_v0;
        echo $__AF_exit23_v0__40 > /dev/null 2>&1
fi
    has_failed__22_v0 "curl -V";
    __AF_has_failed22_v0__44=$__AF_has_failed22_v0;
    if [ $__AF_has_failed22_v0__44 != 0 ]; then
        echo "Curl is not installed on your system."
        echo "Please install \`curl\` and try again."
        exit__23_v0 1;
        __AF_exit23_v0__47=$__AF_exit23_v0;
        echo $__AF_exit23_v0__47 > /dev/null 2>&1
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
            exit__23_v0 1 > /dev/null 2>&1;
            __AF_exit23_v0__63=$__AF_exit23_v0;
            echo $__AF_exit23_v0__63 > /dev/null 2>&1
fi
    if [ ${user_only_install} != 0 ]; then
                    mkdir -p "${bins_folder}" > /dev/null 2>&1
__AS=$?;
if [ $__AS != 0 ]; then
                echo "Failed to create directory for amber bin at ${bins_folder}."
                exit__23_v0 1 > /dev/null 2>&1;
                __AF_exit23_v0__68=$__AF_exit23_v0;
                echo $__AF_exit23_v0__68 > /dev/null 2>&1
fi
fi
    get_latest_release_tag__40_v0 ;
    __AS=$?;
if [ $__AS != 0 ]; then
        echo "Failed to get the latest release tag."
        echo "Please try again or use another download method."
        exit__23_v0 1;
        __AF_exit23_v0__75=$__AF_exit23_v0;
        echo $__AF_exit23_v0__75 > /dev/null 2>&1
fi;
    __AF_get_latest_release_tag40_v0__72="${__AF_get_latest_release_tag40_v0}";
    tag="${__AF_get_latest_release_tag40_v0__72}"
    url="https://github.com/Ph0enixKM/${__0_name}/releases/download/${tag}/amber-${arch}-${os}.tar.xz"
            curl -L -o "${__2_archive}" "${url}" > /dev/null 2>&1
__AS=$?;
if [ $__AS != 0 ]; then
            echo "Curl failed to download amber."
            echo "Something went wrong. Please try again later."
            exit__23_v0 1 > /dev/null 2>&1;
            __AF_exit23_v0__85=$__AF_exit23_v0;
            echo $__AF_exit23_v0__85 > /dev/null 2>&1
fi
    ${sudo} mv "${__2_archive}" "${place}/${__2_archive}"
__AS=$?;
if [ $__AS != 0 ]; then
        echo "Failed to move amber to the installation directory."
        echo "Please make sure that root user can access ${place} directory."
        exit__23_v0 1;
        __AF_exit23_v0__92=$__AF_exit23_v0;
        echo $__AF_exit23_v0__92 > /dev/null 2>&1
fi
            ${sudo} tar --strip-components=1 -xvf ${place}/${__2_archive} -C ${place} > /dev/null 2>&1
__AS=$?;
if [ $__AS != 0 ]; then
            echo "Failed to unarchive amber at ${place}/${__2_archive}"
            echo "Please make sure that you have \`tar\` command installed."
            exit__23_v0 1 > /dev/null 2>&1;
            __AF_exit23_v0__99=$__AF_exit23_v0;
            echo $__AF_exit23_v0__99 > /dev/null 2>&1
fi
    ${sudo} rm ${place}/${__2_archive}
__AS=$?;
if [ $__AS != 0 ]; then
        echo "Failed to remove downloaded archive at ${place}/${__2_archive}"
        exit__23_v0 1;
        __AF_exit23_v0__105=$__AF_exit23_v0;
        echo $__AF_exit23_v0__105 > /dev/null 2>&1
fi
    ${sudo} chmod +x "${place}/${__1_target}"
__AS=$?;
if [ $__AS != 0 ]; then
        echo "Failed to give permissions to execute amber."
        echo "Please make sure that root user can access ${place} directory."
        exit__23_v0 1;
        __AF_exit23_v0__112=$__AF_exit23_v0;
        echo $__AF_exit23_v0__112 > /dev/null 2>&1
fi
    ${sudo} ln -s "${place}/${__1_target}" "${bins_folder}/${__1_target}"
__AS=$?;
if [ $__AS != 0 ]; then
        echo "Failed to create amber symbol link."
        echo "Please make sure that root user can access /usr/local/bin directory."
        exit__23_v0 1;
        __AF_exit23_v0__119=$__AF_exit23_v0;
        echo $__AF_exit23_v0__119 > /dev/null 2>&1
fi
            curl -G --data-urlencode "agent=${__3_agent}" --data-urlencode "name=download" "https://amber-lang.com/api/visit" > /dev/null 2>&1
__AS=$?
    echo "Amber has been installed successfully. 🎉"
    echo "> Now you can use amber by typing \`amber\` in your terminal."
    if [ ${user_only_install} != 0 ]; then
        echo "> Since you requested a user only install with \`--user\` ensure that ~/.local/bin is in your \$PATH."
fi