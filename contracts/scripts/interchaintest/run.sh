source ./utils.sh


DIR="${PWD##*/}"

create_and_run_container $CONFIG_JSON

print "Running scripts..."
npm_store  "$CHAIN_ID_A|$WASM_LENDING_PLATFORM"
npm_inst   "$CHAIN_ID_A|$WASM_LENDING_PLATFORM"
npm_config "$CHAIN_ID_A|$WASM_LENDING_PLATFORM"

CONTAINER_A=$(get_chain_name $CONFIG_JSON $CHAIN_ID_A)
pause_containers $CONTAINER_A