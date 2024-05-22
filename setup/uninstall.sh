function exit__20_v0 {
    local code=$1
            exit "${code}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
:
fi
}
__0_place="/opt/amber";
echo "";

            test -d "${__0_place}" > /dev/null
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
:
fi;
    if [ $(echo $__AMBER_STATUS '==' 0 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//') != 0 ]; then
        sudo rm -rf "${__0_place}"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
            echo "Failed to remove Amber from ${__0_place}";
            echo "Make sure root has the correct permissions to access this directory";
            exit__20_v0 1;
            __AMBER_FUN_exit20_v0__14=${__AMBER_FUN_exit20_v0};
            echo ${__AMBER_FUN_exit20_v0__14} > /dev/null 2>&1
fi;
        sudo rm "/usr/local/bin/amber"
__AMBER_STATUS=$?;
if [ $__AMBER_STATUS != 0 ]; then
            echo "Failed to remove Amber from /usr/local/bin";
            echo "Make sure root has the correct permissions to access this directory";
            exit__20_v0 1;
            __AMBER_FUN_exit20_v0__19=${__AMBER_FUN_exit20_v0};
            echo ${__AMBER_FUN_exit20_v0__19} > /dev/null 2>&1
fi;
        echo "Uninstalled Amber successfully 🎉"
else
        echo "Amber is not installed"
fi