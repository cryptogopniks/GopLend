source ./utils.sh


CONTAINER_A=$(get_chain_name $CONFIG_JSON $CHAIN_ID_A)
stop_and_remove_containers $CONTAINER_A
