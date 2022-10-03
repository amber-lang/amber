__0_name="AmberNative";
__1_target="amber";
__2_tag="v0.1.1-alpha";
__3_place="/opt/amber";
__4_os=$(if [ $([ "_$(uname -s)" != "_Darwin" ]; echo $?) != 0 ]; then echo "macos"; else echo "linux"; fi);
__5_arch=$(if [ $([ "_$(uname -m)" != "_arm64" ]; echo $?) != 0 ]; then echo "aarch64"; else echo "x86_64"; fi);
__6_url="https://github.com/Ph0enixKM/${__0_name}/releases/download/${__2_tag}/amber_${__4_os}_${__5_arch}";
test -d "${__3_place}" > /dev/null;
if [ $([ "_$(echo $?)" != "_0" ]; echo $?) != 0 ]; then
    echo "Amber already installed";
    echo "It seems that Amber is already installed on your system. (${__3_place})";
    echo "If you want to reinstall Amber, uninstall it first.";
    echo "(Find out more at https://amber.marbl.cc)";
    exit 0
fi;
echo "Installing Amber";
if [ $([ "_$(ruby -v > /dev/null; echo $?)" != "_0" ]; echo $?) != 0 ]; then
    __7_code="require \"open-uri\"; open(\"${__1_target}\", \"wb\") do |file|; file << open(\"${__6_url}\").read; end";
    echo "Using ruby as a download method...";
    ruby -e "${__7_code}"
elif [ $([ "_$(curl -v > /dev/null; echo $?)" != "_0" ]; echo $?) != 0 ]; then
    echo "Using curl as a download method...";
    curl -o "${__1_target}" "${__6_url}"
elif [ $([ "_$(wget -v > /dev/null; echo $?)" != "_0" ]; echo $?) != 0 ]; then
    echo "Using wget as a download method...";
    wget -O "${__1_target}" "${__6_url}"
else
    echo "Neither ruby, curl or wget are installed on your system.";
    echo "Please install one of them and try again.";
    exit 1
fi;
sudo mkdir ${__3_place} > /dev/null;
sudo mv ${__1_target} ${__3_place}/${__1_target};
sudo chmod +x ${__3_place}/${__1_target};
sudo ln -s ${__3_place}/${__1_target} /usr/local/bin/${__1_target};
echo "Amber has been installed successfully. 🎉"