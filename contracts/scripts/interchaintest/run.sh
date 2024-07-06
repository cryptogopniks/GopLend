source ./utils.sh


DIR="${PWD##*/}"

create_and_run_container $CONFIG_JSON

print "Running scripts..."
npm_store  "$CHAIN_ID_A|$LABEL_ADAPTER_DEX"
npm_inst   "$CHAIN_ID_A|$LABEL_ADAPTER_DEX"
npm_config "$CHAIN_ID_A|$LABEL_ADAPTER_DEX"

npm_store  "$CHAIN_ID_B|$LABEL_ADAPTER_DEX"
npm_inst   "$CHAIN_ID_B|$LABEL_ADAPTER_DEX"
npm_config "$CHAIN_ID_B|$LABEL_ADAPTER_DEX"

CONTAINER_A=$(get_chain_name $CONFIG_JSON $CHAIN_ID_A)
CONTAINER_B=$(get_chain_name $CONFIG_JSON $CHAIN_ID_B)
CONTAINER_AB=$(get_relayer_name)

pause_containers $CONTAINER_A $CONTAINER_B $CONTAINER_AB