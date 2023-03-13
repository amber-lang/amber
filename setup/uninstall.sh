    place="/opt/amber";
    test -d "${place}" > /dev/null > /dev/null 2>&1;
    if [ $([ "_$(echo $?)" != "_0" ]; echo $?) != 0 ]; then
        sudo rm -rf "${place}" > /dev/null 2>&1;
        sudo rm '/usr/local/bin/amber' > /dev/null 2>&1;
        echo "Uninstalled Amber successfully 🎉"
else
        echo "Amber is not installed"
fi