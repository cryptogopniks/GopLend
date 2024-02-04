# Example of calling a contract using daemon CLI

source ./utils.sh

RPC_A="http://localhost:26657/"
RPC_B="http://localhost:26657/"

ACCOUNT_A="stars19y7a38cnf9d8cr264wz5d6dmrsgsmplxkf4lyw"

TXFLAG_A="-y --gas auto --gas-prices 0.1ustars --gas-adjustment 1.3 --chain-id $CHAIN_ID_A --node $RPC_A"

QFLAG_A="--chain-id $CHAIN_ID_A --node $RPC_A"

DIR="${PWD##*/}"


CONTAINER_A=$(get_chain_name $CONFIG_JSON $CHAIN_ID_A)

DAEMON_A="docker exec -i $CONTAINER_A starsd"

unpause_containers $CONTAINER_A

print "Running scripts..."

# echo 'enter seed phrase "lemon degree side riot dawn spice oblige patch six cross judge grid"'
# echo "then enter keyring password 12345678 twice"
# $DAEMON_A keys add "admin" --recover


pause_containers $CONTAINER_A
