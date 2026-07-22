if [ -n "$ZSH_VERSION" ]; then
    EXEC_SHELL="zsh"
elif [ -n "$KSH_VERSION" ]; then
    EXEC_SHELL="ksh"
else
    EXEC_SHELL="bash"
fi
