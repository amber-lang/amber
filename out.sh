tag="1.0.0";
place="/opt/amber";
url="https://github.com/Ph0enixKM/Amber/releases/download/$tag/amber.zip";
test -d "$place" > /dev/null;
if [ $([ "_$(echo $?)" != "_"0"" ]; echo $?) != 0 ]; then
echo "Amber already installed" ;
echo "It seems that Amber is already installed on your system.";
echo "If you want to reinstall Amber - try to uninstall it first.";
echo "(Find out more at https://amber.marbl.cc)";
exit 0
fi;
echo "Installing Amber" ;
target="amber.zip";
if [ $([ "_$(ruby -v > /dev/null; echo $?)" != "_"0"" ]; echo $?) != 0 ]; then
code="require \"open-uri\"; open(\"$target\", \"wb\") do |file|; file << open(\"$url\").read; end";
echo "Using ruby as a download method...";
ruby -e "$code"
elif [ $([ "_$(curl -v > /dev/null; echo $?)" != "_"0"" ]; echo $?) != 0 ]; then
echo "Using curl as a download method...";
curl -o "$target" "$url"
elif [ $([ "_$(wget -v > /dev/null; echo $?)" != "_"0"" ]; echo $?) != 0 ]; then
echo "Using wget as a download method...";
wget -O "$target" "$url"
else
echo "Neither ruby, curl or wget are installed on your system.";
echo "Please install one of them and try again.";
exit 1
fi;
sudo mkdir $place > /dev/null;
sudo mv amber.zip $place/amber.zip;
pushd $place > /dev/null;
echo 'Unpacking...';
sudo unzip $place/amber.zip;
sudo rm $place/amber.zip;
sudo ln -s $place/amber /usr/local/bin/amber;
popd $place;
echo "Amber has been installed successfully."