    place="/opt/amber";
    test -d "${place}" > /dev/null
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
$(exit $__AMBER_STATUS)
:
fi;
    __AMBER_VAL_0=$(echo $?);
    __AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
$(exit $__AMBER_STATUS)
:
fi;
    if [ $([ "_${__AMBER_VAL_0}" != "_0" ]; echo $?) != 0 ]; then
        sudo rm -rf "${place}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
$(exit $__AMBER_STATUS)
:
fi;
        sudo rm "/usr/local/bin/amber"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
$(exit $__AMBER_STATUS)
:
fi;
        echo "Uninstalled Amber successfully 🎉"
else
        echo "Amber is not installed"
fi