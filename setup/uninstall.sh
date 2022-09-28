place="/opt/amber";
test -d "${place}" > /dev/null;
if [ $([ "_$(echo $?)" != "_0" ]; echo $?) != 0 ]; then
    sudo rm -rf "${place}";
    sudo rm '/usr/local/bin/amber';
    echo 'Uninstalled Amber successfully 🎉'
else
    echo 'Amber is not installed'
fi