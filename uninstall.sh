#!/bin/bash
amber_path="./link.sh"

if [ -z "$amber_path" ]; then
    echo "Amber is not in \$PATH."
    exit 1
fi

amber_installation_dir=$(dirname "$(readlink -f "$amber_path")")

echo "Uninstalling Amber..."
rm -rf $amber_path
rm -rf $amber_installation_dir
echo "Amber uninstalled. See you later, 🐊"
