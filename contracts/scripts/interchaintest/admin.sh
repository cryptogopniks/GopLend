source ./utils.sh

CONTAINER_A=$(get_chain_name $CONFIG_JSON $CHAIN_ID_A)
DAEMON_A="docker exec -i $CONTAINER_A starsd"


unpause_containers $CONTAINER_A

echo 'enter seed phrase "lemon degree side riot dawn spice oblige patch six cross judge grid"'
echo "then enter keyring password 12345678 twice"
$DAEMON_A keys add "admin" --recover

pause_containers $CONTAINER_A
