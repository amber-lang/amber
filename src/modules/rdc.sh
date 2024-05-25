AMBER_RDC_MD=()
for d in "${AMBER_RDC_CD[@]}"
do
    if ! command -v $d > /dev/null 2>&1; then
        AMBER_RDC_MD+=($d)
    fi
done

if (( ${#AMBER_RDC_MD[@]} != 0 )); then
    >&2 echo This program requires for these commands: \( $AMBER_RDC_MD \) to be present in \$PATH.
    exit 1
fi
unset $AMBER_RDC_CD
unset $AMBER_RDC_MD
# Dependencies are ok at this point
