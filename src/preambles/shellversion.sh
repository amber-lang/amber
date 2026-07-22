if [ -n "$ZSH_VERSION" ]; then
    __exec_shell_version="$ZSH_VERSION"
    IFS='.' read -r __exec_v1 __exec_v2 __exec_v3 <<< "${__exec_shell_version}"
    set -A EXEC_SHELL_VERSION -- "${__exec_v1:-0}" "${__exec_v2:-0}" "${__exec_v3:-0}"
elif [ -n "$KSH_VERSION" ]; then
    __exec_shell_version="${KSH_VERSION#Version }"
    __exec_shell_version="${__exec_shell_version#* }"
    __exec_shell_version="${__exec_shell_version%% *}"
    __exec_v1="${__exec_shell_version%%[!0-9]*}"
    __exec_v2=0
    __exec_v3=0
    EXEC_SHELL_VERSION=(${__exec_v1:-0} ${__exec_v2:-0} ${__exec_v3:-0})
else
    EXEC_SHELL_VERSION=("${BASH_VERSINFO[0]}" "${BASH_VERSINFO[1]}" "${BASH_VERSINFO[2]}")
fi
