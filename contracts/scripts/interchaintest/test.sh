source ./utils.sh

DIR="${PWD##*/}"


CONTAINER_A=$(get_chain_name $CONFIG_JSON $CHAIN_ID_A)
unpause_containers $CONTAINER_A

print "Running scripts..."
# npm_store  "$CHAIN_ID_A|$WASM_LENDING_PLATFORM"
# npm_inst   "$CHAIN_ID_A|$WASM_LENDING_PLATFORM"
# npm_config "$CHAIN_ID_A|$WASM_LENDING_PLATFORM"
npm_call     "$CHAIN_ID_A|"

pause_containers $CONTAINER_A $CONTAINER_B $CONTAINER_AB
