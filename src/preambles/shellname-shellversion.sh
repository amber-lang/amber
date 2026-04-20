if [ -n "$ZSH_VERSION" ]; then
    EXEC_SHELL="zsh"
    IFS='.' read -A EXEC_SHELL_VERSION <<< "$ZSH_VERSION"
elif [ -n "$KSH_VERSION" ]; then
    EXEC_SHELL="ksh"
    # In ksh, parse VERSION variable directly (format: "Version AJM 93u+ 2012-08-01")
    _ksh_ver="${VERSION:-}"
    if [ -n "$_ksh_ver" ]; then
        EXEC_SHELL_VERSION=($_ksh_ver)
    else
        EXEC_SHELL_VERSION=("unknown" "unknown" "unknown")
    fi
else
    EXEC_SHELL="bash"
    EXEC_SHELL_VERSION=("${BASH_VERSINFO[0]}" "${BASH_VERSINFO[1]}" "${BASH_VERSINFO[2]}")
fi