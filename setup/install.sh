__0_name="AmberNative";
__1_target="amber";
__2_tag="v0.1.1-alpha";
__3_place="/opt/amber";
    os=$(if [ $([ "_$(uname -s)" != "_Darwin" ]; echo $?) != 0 ]; then echo "macos"; else echo "linux"; fi);
    arch=$(if [ $([ "_$(uname -m)" != "_arm64" ]; echo $?) != 0 ]; then echo "aarch64"; else echo "x86_64"; fi);
    url="https://github.com/Ph0enixKM/${__0_name}/releases/download/${__2_tag}/amber_${os}_${arch}";
    test -d "${__3_place}" > /dev/null 2>&1;
    if [ $([ "_$(echo $?)" != "_0" ]; echo $?) != 0 ]; then
        echo "Amber already installed";
        echo "It seems that Amber is already installed on your system. (${__3_place})";
        echo "If you want to reinstall Amber, uninstall it first.";
        echo "(Find out more at https://amber.marbl.cc)";
        exit 0 > /dev/null 2>&1
fi;
    echo "Installing Amber";
    if [ $([ "_$(ruby -v > /dev/null; echo $?)" != "_0" ]; echo $?) != 0 ]; then
        code="require \"open-uri\"; open(\"${__1_target}\", \"wb\") do |file|; file << open(\"${url}\").read; end";
        echo "Using ruby as a download method...";
        ruby -e "${code}" > /dev/null 2>&1
elif [ $([ "_$(curl -v > /dev/null; echo $?)" != "_0" ]; echo $?) != 0 ]; then
        echo "Using curl as a download method...";
        curl -o "${__1_target}" "${url}" > /dev/null 2>&1
elif [ $([ "_$(wget -v > /dev/null; echo $?)" != "_0" ]; echo $?) != 0 ]; then
        echo "Using wget as a download method...";
        wget -O "${__1_target}" "${url}" > /dev/null 2>&1
else
        echo "Neither ruby, curl or wget are installed on your system.";
        echo "Please install one of them and try again.";
        exit 1 > /dev/null 2>&1
fi;
    sudo mkdir "${__3_place}" > /dev/null > /dev/null 2>&1;
    sudo mv "${__1_target}" "${__3_place}/${__1_target}" > /dev/null 2>&1;
    sudo chmod +x "${__3_place}/${__1_target}" > /dev/null 2>&1;
    sudo ln -s "${__3_place}/${__1_target}" "/usr/local/bin/${__1_target}" > /dev/null 2>&1;
    echo "Amber has been installed successfully. 🎉"