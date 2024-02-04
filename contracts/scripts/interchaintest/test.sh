source ./utils.sh

DIR="${PWD##*/}"


CONTAINER_A=$(get_chain_name $CONFIG_JSON $CHAIN_ID_A)
unpause_containers $CONTAINER_A

print "Running scripts..."
npm_call

pause_containers $CONTAINER_A $CONTAINER_B $CONTAINER_AB
