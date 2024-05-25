MD=()
for d in "${CD[@]}"
do
    if ! command -v $d > /dev/null 2>&1; then
        MD+=($d)
    fi
done

if (( ${#MD[@]} != 0 )); then
    >&2 echo This program requires for these commands: \( $MD \) to be present in \$PATH.
    exit 1
fi
unset $CD
unset $MD
# Dependencies are ok at this point
