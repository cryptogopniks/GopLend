# Example of calling a contract using daemon CLI

source ./utils.sh

RPC_A="http://localhost:26657/"
RPC_B="http://localhost:26657/"

ACCOUNT_A="stars19y7a38cnf9d8cr264wz5d6dmrsgsmplxkf4lyw"
ACCOUNT_B="stars10d07y265gmmuvt4z0w9aw880jnsr700jw7ycaz"

TXFLAG_A="-y --gas auto --gas-prices 0.1ustars --gas-adjustment 1.3 --chain-id $CHAIN_ID_A --node $RPC_A"

QFLAG_A="--chain-id $CHAIN_ID_A --node $RPC_A"

DIR="${PWD##*/}"

FUNDS="1ustars"
PASSWORD="12345678"
CONTRACT_ADDRESS="stars1ufs3tlq4umljk0qfe8k5ya0x6hpavn897u2cnf9k0en9jr7qarqqan7eqa"

CONTAINER_A=$(get_chain_name $CONFIG_JSON $CHAIN_ID_A)

DAEMON_A="docker exec -i $CONTAINER_A starsd"

PROPOSAL="{\"messages\":[{\"@type\":\"/publicawesome.stargaze.cron.v1.MsgPromoteToPrivilegedContract\",\"authority\":\"stars10d07y265gmmuvt4z0w9aw880jnsr700jw7ycaz\",\"contract\":\"stars1ufs3tlq4umljk0qfe8k5ya0x6hpavn897u2cnf9k0en9jr7qarqqan7eqa\"}],\"metadata\":\"metadata\",\"deposit\":\"1ustars\",\"title\":\"Promote contract\",\"summary\":\"Contract will get begin and end blocker callbacks\"}"

unpause_containers $CONTAINER_A

print "Running scripts..."

# echo 'enter seed phrase "lemon degree side riot dawn spice oblige patch six cross judge grid"'
# echo "then enter keyring password 12345678 twice"
# $DAEMON_A keys add "admin" --recover

yes $PASSWORD | $DAEMON_A tx gov submit-proposal promote-to-privilege-contract $CONTRACT_ADDRESS \
  --title "contract promotion" \
  --description="promote lending contract" \
  --deposit $FUNDS \
  --from $ACCOUNT_A \
  $TXFLAG_A

npm_gov "$CHAIN_ID_A|$WASM_LENDING_PLATFORM"

$DAEMON_A q cron list-privileged $QFLAG_A


pause_containers $CONTAINER_A
