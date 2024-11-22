#!/usr/bin/env bash
# Written in [Amber](https://amber-lang.com/)
# version: 0.3.5-alpha
# date: 2024-11-22 12:46:50
file_exist__34_v0() {
    local path=$1
    [ -f "${path}" ]
    __AS=$?
    if [ $__AS != 0 ]; then
        __AF_file_exist34_v0=0
        return 0
    fi
    __AF_file_exist34_v0=1
    return 0
}
input__95_v0() {
    local prompt=$1
    read -p "$prompt"
    __AS=$?
    __AF_input95_v0="$REPLY"
    return 0
}
has_failed__98_v0() {
    local command=$1
    eval ${command} >/dev/null 2>&1
    __AS=$?
    __AF_has_failed98_v0=$(echo $__AS '!=' 0 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//')
    return 0
}
array_first_index__114_v0() {
    local array=("${!1}")
    local value=$2
    index=0
    for element in "${array[@]}"; do
        if [ $(
            [ "_${value}" != "_${element}" ]
            echo $?
        ) != 0 ]; then
            __AF_array_first_index114_v0=${index}
            return 0
        fi
        ((index++)) || true
    done
    __AF_array_first_index114_v0=$(echo '-' 1 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//')
    return 0
}
includes__116_v0() {
    local array=("${!1}")
    local value=$2
    array_first_index__114_v0 array[@] "${value}"
    __AF_array_first_index114_v0__26_18="$__AF_array_first_index114_v0"
    local result="$__AF_array_first_index114_v0__26_18"
    __AF_includes116_v0=$(echo ${result} '>=' 0 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//')
    return 0
}
get_os__121_v0() {
    # Determine OS type
    __AMBER_VAL_0=$(uname -s)
    __AS=$?
    if [ $__AS != 0 ]; then
        echo "Failed to determine OS type (using \`uname\` command)."
        echo "Please try again or make sure you have it installed."
        exit 1
    fi
    local os_type="${__AMBER_VAL_0}"
    if [ $(
        [ "_${os_type}" != "_Darwin" ]
        echo $?
    ) != 0 ]; then
        __AF_get_os121_v0="apple-darwin"
        return 0
    fi
    if [ $(
        [ "_${os_type}" == "_Linux" ]
        echo $?
    ) != 0 ]; then
        echo "Unsupported OS type: ${os_type}"
        echo "Please try again or use another download method."
        exit 1
    fi
    has_failed__98_v0 "ls -l /lib | grep libc.musl"
    __AF_has_failed98_v0__20_12="$__AF_has_failed98_v0"
    if [ $(echo '!' "$__AF_has_failed98_v0__20_12" | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//') != 0 ]; then
        __AF_get_os121_v0="unknown-linux-musl"
        return 0
    fi
    __AF_get_os121_v0="unknown-linux-gnu"
    return 0
}
get_arch__122_v0() {
    # Determine architecture
    __AMBER_VAL_1=$(uname -m)
    __AS=$?
    if [ $__AS != 0 ]; then
        echo "Failed to determine architecture."
        echo "Please try again or use another download method."
        exit 1
    fi
    local arch_type="${__AMBER_VAL_1}"
    __AMBER_ARRAY_0=("arm64" "aarch64")
    includes__116_v0 __AMBER_ARRAY_0[@] "${arch_type}"
    __AF_includes116_v0__34_16="$__AF_includes116_v0"
    local arch=$(if [ "$__AF_includes116_v0__34_16" != 0 ]; then echo "aarch64"; else echo "x86_64"; fi)
    __AF_get_arch122_v0="${arch}"
    return 0
}
get_home__123_v0() {
    __AMBER_VAL_2=$(echo $HOME)
    __AS=$?
    if [ $__AS != 0 ]; then
        echo "User installation requested, but unable to retrieve home directory from \$HOME environment."
        exit 1
    fi
    local home="${__AMBER_VAL_2}"
    if [ $(
        [ "_${home}" != "_" ]
        echo $?
    ) != 0 ]; then
        echo "User installation requested, but unable to find home directory."
        exit 1
    fi
    __AF_get_home123_v0="${home}"
    return 0
}
get_bins_folder__124_v0() {
    local user_only=$1
    if [ ${user_only} != 0 ]; then
        get_home__123_v0
        __AF_get_home123_v0__55_18="${__AF_get_home123_v0}"
        __AF_get_bins_folder124_v0="${__AF_get_home123_v0__55_18}/.local/bin"
        return 0
    else
        # Ensure /usr/local/bin exists for non-user installations
        local bins_folder="/usr/local/bin"
        test -d "${bins_folder}" >/dev/null 2>&1
        __AS=$?
        if [ $__AS != 0 ]; then
            sudo mkdir -p "${bins_folder}" >/dev/null 2>&1
            __AS=$?
            if [ $__AS != 0 ]; then
                echo "Failed to create ${bins_folder} directory."
                exit 1
            fi
        fi
        __AF_get_bins_folder124_v0="${bins_folder}"
        return 0
    fi
}
get_place__125_v0() {
    local user_only=$1
    if [ ${user_only} != 0 ]; then
        get_home__123_v0
        __AF_get_home123_v0__71_18="${__AF_get_home123_v0}"
        get_arch__122_v0
        __AF_get_arch122_v0__71_42="${__AF_get_arch122_v0}"
        __AF_get_place125_v0="${__AF_get_home123_v0__71_18}/.local/lib/${__AF_get_arch122_v0__71_42}/amber"
        return 0
    else
        __AF_get_place125_v0="/opt/amber"
        return 0
    fi
}
__0_name="amber"
__1_target="amber"
__2_archive="amber.tar.xz"
has_failed__98_v0 "uname -a"
__AF_has_failed98_v0__10_13="$__AF_has_failed98_v0"
__AMBER_VAL_3=$(uname -a)
__AS=$?
__3_agent=$(if [ "$__AF_has_failed98_v0__10_13" != 0 ]; then echo "unknown"; else echo "${__AMBER_VAL_3}"; fi)
echo ""
get_latest_release_tag__130_v0() {
    local tag_url="https://api.github.com/repos/amber-lang/${__0_name}/releases/latest"
    __AMBER_VAL_4=$(curl -sL "${tag_url}")
    __AS=$?
    if [ $__AS != 0 ]; then
        __AF_get_latest_release_tag130_v0=''
        return $__AS
    fi
    local tag_json="${__AMBER_VAL_4}"
    # Get the tag name from the JSON
    __AMBER_VAL_5=$(echo "$tag_json" | grep -Eo "tag_name\"[^\"]*\"([^\"]+)\"" | grep -Eo "\"[^\"]+\"$" | grep -Eo "[^\"\s]+")
    __AS=$?
    if [ $__AS != 0 ]; then
        __AF_get_latest_release_tag130_v0=''
        return $__AS
    fi
    local tag="${__AMBER_VAL_5}"
    __AF_get_latest_release_tag130_v0="${tag}"
    return 0
}
args=("$0" "$@")
get_os__121_v0
__AF_get_os121_v0__28_14="${__AF_get_os121_v0}"
os="${__AF_get_os121_v0__28_14}"
get_arch__122_v0
__AF_get_arch122_v0__29_16="${__AF_get_arch122_v0}"
arch="${__AF_get_arch122_v0__29_16}"
includes__116_v0 args[@] "--user"
__AF_includes116_v0__31_29="$__AF_includes116_v0"
user_only_install="$__AF_includes116_v0__31_29"
get_place__125_v0 ${user_only_install}
__AF_get_place125_v0__32_17="${__AF_get_place125_v0}"
place="${__AF_get_place125_v0__32_17}"
get_bins_folder__124_v0 ${user_only_install}
__AF_get_bins_folder124_v0__33_23="${__AF_get_bins_folder124_v0}"
bins_folder="${__AF_get_bins_folder124_v0__33_23}"
# Check if such directory exists
test -d "${place}"
__AS=$?
if [ $(echo $__AS '==' 0 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//') != 0 ]; then
    echo "Amber already installed"
    echo "It seems that Amber is already installed on your system. (${place})"
    echo "If you want to reinstall Amber, uninstall it first."
    echo "(Find out more at https://docs.amber-lang.com/getting_started/installation#uninstallation)"
    exit 2
fi
# Check if curl is installed
has_failed__98_v0 "curl -V"
__AF_has_failed98_v0__47_8="$__AF_has_failed98_v0"
if [ "$__AF_has_failed98_v0__47_8" != 0 ]; then
    echo "Curl is not installed on your system."
    echo "Please install \`curl\` and try again."
    exit 1
fi
echo "Installing Amber... 🚀"
# Make the directories we need first to ensure we have permissions before downloading any files
# this decreases the chance that our script results in partial installation leaving assets behind
sudo=$(if [ ${user_only_install} != 0 ]; then echo ""; else echo "sudo"; fi)
# Create directory for amber
${sudo} mkdir -p "${place}" >/dev/null 2>&1
__AS=$?
if [ $__AS != 0 ]; then
    echo "Failed to create directory for amber."
    if [ ${user_only_install} != 0 ]; then
        echo "Please make sure that root user can access ${place} directory."
    else
        echo "Please make sure that your user can access ${place} directory."
    fi
    exit 1
fi
if [ ${user_only_install} != 0 ]; then
    mkdir -p "${bins_folder}" >/dev/null 2>&1
    __AS=$?
    if [ $__AS != 0 ]; then
        echo "Failed to create directory for amber bin at ${bins_folder}."
        exit 1
    fi
fi
get_latest_release_tag__130_v0
__AS=$?
if [ $__AS != 0 ]; then
    echo "Failed to get the latest release tag."
    echo "Please try again or use another download method."
    exit 1
fi
__AF_get_latest_release_tag130_v0__75_15="${__AF_get_latest_release_tag130_v0}"
tag="${__AF_get_latest_release_tag130_v0__75_15}"
# Set the download link
url="https://github.com/amber-lang/${__0_name}/releases/download/${tag}/amber-${arch}-${os}.tar.xz"
# Download amber
curl -L -o "${__2_archive}" "${url}" >/dev/null 2>&1
__AS=$?
if [ $__AS != 0 ]; then
    echo "Curl failed to download amber."
    echo "Something went wrong. Please try again later."
    exit 1
fi
# Move archived version of amber
${sudo} mv "${__2_archive}" "${place}/${__2_archive}"
__AS=$?
if [ $__AS != 0 ]; then
    echo "Failed to move amber to the installation directory."
    echo "Please make sure that root user can access ${place} directory."
    exit 1
fi
# Unarchive amber
${sudo} tar --strip-components=1 -xvf ${place}/${__2_archive} -C ${place} >/dev/null 2>&1
__AS=$?
if [ $__AS != 0 ]; then
    echo "Failed to unarchive amber at ${place}/${__2_archive}"
    echo "Please make sure that you have \`tar\` command installed."
    exit 1
fi
# Delete the archive
${sudo} rm ${place}/${__2_archive}
__AS=$?
if [ $__AS != 0 ]; then
    echo "Failed to remove downloaded archive at ${place}/${__2_archive}"
    exit 1
fi
# Give permissions to execute amber
${sudo} chmod +x "${place}/${__1_target}"
__AS=$?
if [ $__AS != 0 ]; then
    echo "Failed to give permissions to execute amber."
    echo "Please make sure that root user can access ${place} directory."
    exit 1
fi
# Delete the previous symbolic link
file_exist__34_v0 "${bins_folder}/${__1_target}"
__AF_file_exist34_v0__119_8="$__AF_file_exist34_v0"
if [ "$__AF_file_exist34_v0__119_8" != 0 ]; then
    ${sudo} rm "${bins_folder}/${__1_target}"
    __AS=$?
    if [ $__AS != 0 ]; then
        echo "Failed to remove the previous amber symbol link."
        echo "Please make sure that root user can access ${bins_folder} directory."
        exit 1
    fi
fi
# Create amber symbol link
${sudo} ln -s "${place}/${__1_target}" "${bins_folder}/${__1_target}"
__AS=$?
if [ $__AS != 0 ]; then
    echo "Failed to create amber symbol link."
    echo "Please make sure that root user can access ${bins_folder} directory."
    exit 1
fi
input__95_v0 "Would you like to help improve Amber by sharing your OS info with our developer database? Enter your GitHub nickname (or any nickname) or type \`no\`:"
__AF_input95_v0__134_20="${__AF_input95_v0}"
nickname="${__AF_input95_v0__134_20}"
if [ $(
    [ "_${nickname}" == "_no" ]
    echo $?
) != 0 ]; then
    # Send feedback to the server
    curl -G --data-urlencode "agent=${__3_agent}" --data-urlencode "nickname=${nickname}" --data-urlencode "name=download" "https://amber-lang.com/api/visit" >/dev/null 2>&1
    __AS=$?
fi
# Send success message
echo "Amber has been installed successfully. 🎉"
echo "> Now you can use amber by typing \`amber\` in your terminal."
if [ ${user_only_install} != 0 ]; then
    echo "> Since you requested a user only install with \`--user\` ensure that ~/.local/bin is in your \\\$PATH."
fi
