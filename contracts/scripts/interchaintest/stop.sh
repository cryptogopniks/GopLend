source ./utils.sh


CONTAINER_A=$(get_chain_name $CONFIG_JSON $CHAIN_ID_A)
CONTAINER_B=$(get_chain_name $CONFIG_JSON $CHAIN_ID_B)
CONTAINER_AB=$(get_relayer_name)

stop_and_remove_containers $CONTAINER_A $CONTAINER_B $CONTAINER_AB
