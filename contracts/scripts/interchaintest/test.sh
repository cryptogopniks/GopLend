source ./utils.sh

DIR="${PWD##*/}"


CONTAINER_A=$(get_chain_name $CONFIG_JSON $CHAIN_ID_A)
CONTAINER_B=$(get_chain_name $CONFIG_JSON $CHAIN_ID_B)
CONTAINER_AB=$(get_relayer_name)
unpause_containers $CONTAINER_A $CONTAINER_B $CONTAINER_AB

print "Running scripts..."
# npm_store  "$CHAIN_ID_A|$LABEL_ADAPTER_DEX"
# npm_inst   "$CHAIN_ID_A|$LABEL_ADAPTER_DEX"
# npm_config "$CHAIN_ID_A|$LABEL_ADAPTER_DEX"
npm_call     "$CHAIN_ID_A|"

pause_containers $CONTAINER_A $CONTAINER_B $CONTAINER_AB
