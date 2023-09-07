function hasFailed__18_v0 {
    local command=$1
            ${command} > /dev/null 2>&1
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
$(exit $__AMBER_STATUS)
:
fi;
    __AMBER_FUN_hasFailed18_v0=$(echo $__AMBER_STATUS '!=' 0 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//');
    return 0
}
__0_name="AmberNative";
__1_target="amber";
__2_tag="0.3.1-alpha";
__3_place="/opt/amber";
    __AMBER_VAL_0=$(uname -s);
    __AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
$(exit $__AMBER_STATUS)
:
fi;
    os=$(if [ $([ "_${__AMBER_VAL_0}" != "_Darwin" ]; echo $?) != 0 ]; then echo "macos"; else echo "linux"; fi);
    __AMBER_VAL_1=$(uname -m);
    __AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
$(exit $__AMBER_STATUS)
:
fi;
    arch=$(if [ $([ "_${__AMBER_VAL_1}" != "_arm64" ]; echo $?) != 0 ]; then echo "aarch64"; else echo "x86_64"; fi);
    url="https://github.com/Ph0enixKM/${__0_name}/releases/download/${__2_tag}/amber_${os}_${arch}";
    test -d "${__3_place}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
$(exit $__AMBER_STATUS)
:
fi;
    if [ $(echo $__AMBER_STATUS '==' 0 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//') != 0 ]; then
        echo "Amber already installed";
        echo "It seems that Amber is already installed on your system. (${__3_place})";
        echo "If you want to reinstall Amber, uninstall it first.";
        echo "(Find out more at https://amber.marbl.cc)";
        exit 0
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
$(exit $__AMBER_STATUS)
:
fi
fi;
    echo "Installing Amber";
    hasFailed__18_v0 "ruby -v";
    __AMBER_FUN_hasFailed18_v0__36 = __AMBER_FUN_hasFailed18_v0;
    hasFailed__18_v0 "curl -v";
    __AMBER_FUN_hasFailed18_v0__41 = __AMBER_FUN_hasFailed18_v0;
    hasFailed__18_v0 "wget -V";
    __AMBER_FUN_hasFailed18_v0__45 = __AMBER_FUN_hasFailed18_v0;
    if [ $(echo  '!' ${__AMBER_FUN_hasFailed18_v0__36} | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//') != 0 ]; then
        code="require \"open-uri\"; open(\"${__1_target}\", \"wb\") do |file|; file << open(\"${url}\").read; end";
        echo "Using ruby as a download method...";
        sudo ruby -e "${code}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
$(exit $__AMBER_STATUS)
:
fi
elif [ $(echo  '!' ${__AMBER_FUN_hasFailed18_v0__41} | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//') != 0 ]; then
        echo "Using curl as a download method...";
        curl -o "${__1_target}" "${url}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
$(exit $__AMBER_STATUS)
:
fi
elif [ $(echo  '!' ${__AMBER_FUN_hasFailed18_v0__45} | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//') != 0 ]; then
        echo "Using wget as a download method...";
        wget -O "${__1_target}" "${url}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
$(exit $__AMBER_STATUS)
:
fi
else
        echo "Neither ruby, curl or wget are installed on your system.";
        echo "Please install one of them and try again.";
        exit 1
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
$(exit $__AMBER_STATUS)
:
fi
fi;
    sudo mkdir "${__3_place}" > /dev/null
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
$(exit $__AMBER_STATUS)
:
fi;
    sudo mv "${__1_target}" "${__3_place}/${__1_target}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
$(exit $__AMBER_STATUS)
:
fi;
    sudo chmod +x "${__3_place}/${__1_target}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
$(exit $__AMBER_STATUS)
:
fi;
    sudo ln -s "${__3_place}/${__1_target}" "/usr/local/bin/${__1_target}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
$(exit $__AMBER_STATUS)
:
fi;
    echo "Amber has been installed successfully. 🎉"