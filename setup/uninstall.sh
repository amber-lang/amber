    place="/opt/amber";
    test -d "${place}" > /dev/null
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
:
fi;
    __AMBER_VAL_0=$(echo $?);
    __AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
:
fi;
    if [ $([ "_${__AMBER_VAL_0}" != "_0" ]; echo $?) != 0 ]; then
        sudo rm -rf "${place}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
:
fi;
        sudo rm "/usr/local/bin/amber"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
:
fi;
        echo "Uninstalled Amber successfully 🎉"
else
        echo "Amber is not installed"
fi