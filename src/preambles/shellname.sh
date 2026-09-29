if [ -n "$ZSH_VERSION" ]; then
    __EXEC_SHELL="zsh"
elif [ -n "$KSH_VERSION" ]; then
    __EXEC_SHELL="ksh"
else
    __EXEC_SHELL="bash"
fi
