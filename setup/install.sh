name="AmberNative";
target="amber";
tag="v0.1.1-alpha";
place="/opt/amber";
os=$(if [ $([ "_$(uname -s)" != "_Darwin" ]; echo $?) != 0 ]; then echo "macos"; else echo "linux"; fi);
arch=$(if [ $([ "_$(uname -m)" != "_arm64" ]; echo $?) != 0 ]; then echo "aarch64"; else echo "x86_64"; fi);
url="https://github.com/Ph0enixKM/${name}/releases/download/${tag}/amber_${os}_${arch}";
test -d "${place}" > /dev/null;
if [ $([ "_$(echo $?)" != "_0" ]; echo $?) != 0 ]; then
    echo "Amber already installed" ;
    echo "It seems that Amber is already installed on your system.";
    echo "If you want to reinstall Amber - try to uninstall it first.";
    echo "(Find out more at https://amber.marbl.cc)";
    exit 0
fi;
echo "Installing Amber" ;
if [ $([ "_$(ruby -v > /dev/null; echo $?)" != "_0" ]; echo $?) != 0 ]; then
    code="require \"open-uri\"; open(\"${target}\", \"wb\") do |file|; file << open(\"${url}\").read; end";
    echo "Using ruby as a download method...";
    ruby -e "${code}"
elif [ $([ "_$(curl -v > /dev/null; echo $?)" != "_0" ]; echo $?) != 0 ]; then
    echo "Using curl as a download method...";
    curl -o "${target}" "${url}"
elif [ $([ "_$(wget -v > /dev/null; echo $?)" != "_0" ]; echo $?) != 0 ]; then
    echo "Using wget as a download method...";
    wget -O "${target}" "${url}"
else
    echo "Neither ruby, curl or wget are installed on your system.";
    echo "Please install one of them and try again.";
    exit 1
fi;
sudo mkdir ${place} > /dev/null;
sudo mv ${target} ${place}/${target};
sudo chmod +x ${place}/${target};
sudo ln -s ${place}/${target} /usr/local/bin/${target};
echo "Amber has been installed successfully. 🎉"