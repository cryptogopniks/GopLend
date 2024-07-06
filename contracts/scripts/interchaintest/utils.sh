CONFIG_JSON="./chains/stargaze_terra.json"
CHAIN_ID_A="stargaze-0"
CHAIN_ID_B="terra-0"
LABEL_ADAPTER_DEX="adapter_dex"


function print() {
    echo "------------------------------------------------------------------------------------"
    echo "$1"
    echo
}

function npm_json() {
  local dir="${PWD##*/}"

  cd ..
  npm run json
  cd $dir
}

function npm_logs() {
  local dir="${PWD##*/}"

  cd ..
  npm run logs
  cd $dir
}

function npm_store() {
  local args="$1"
  local dir="${PWD##*/}"

  cd ..
  npm run store $args
  cd $dir
}

function npm_gov() {
  local args="$1"
  local dir="${PWD##*/}"

  cd ..
  npm run gov $args
  cd $dir
}

function npm_migrate() {
  local args="$1"
  local dir="${PWD##*/}"

  cd ..
  npm run migrate $args
  cd $dir
}

function npm_inst() {
  local args="$1"
  local dir="${PWD##*/}"

  cd ..
  npm run inst $args
  cd $dir
}

function npm_config() {
  local args="$1"
  local dir="${PWD##*/}"

  cd ..
  npm run config $args
  cd $dir
}

function npm_call() {
  local args="$1"
  local dir="${PWD##*/}"

  cd ..
  npm run call $args
  cd $dir
}

function create_and_run_container() {
  local json_config="$1"

  local path_to_chains="$HOME/dev/interchaintest/local-interchain/chains"
  local logs="$HOME/dev/interchaintest/local-interchain/configs/logs.json"
  local path_to_logs_1="./configs"
  local path_to_logs_2="../src/common/config"
  local port="8080"

  # Use parameter expansion to remove everything 
  # before final / and after .json
  filename="${json_config##*/}"
  filename="${filename%.*}"

  print "Starting container..."
  cp $json_config $path_to_chains
  local-ic start "$filename" --api-port $port &> /dev/null &
  
  # Wait for API server
  local url="http://localhost:$port"
  
  while [[ "$(curl -s -o /dev/null -w ''%{http_code}'' ${url})" != "200" ]]; do
    sleep 1
  done

  # Save new endpoints
  while true; do
    content=$(cat $logs)
    length=${#content}
    if [ $length -ge 5 ]; then
      break
    fi
    sleep 0.5
  done

  cp $logs $path_to_logs_1
  cp $logs $path_to_logs_2

  npm_logs
}

function get_chain_name() {
  local json_config="$1"
  local chain_id="$2"

  # Extract image name
  local image_name=$(jq -r ".chains[] | select(.chain_id == \"$chain_id\") | .docker_image.repository + \":\" + .docker_image.version" $json_config)

  # Get all container IDs for image
  local container_ids=$(docker ps -aq --filter ancestor=$image_name)

  # Loop through IDs and print names
  for id in $container_ids; do
    local constainer_name=$(docker inspect -f '{{.Name}}' $id | sed 's/\///g')
  done

  echo $constainer_name
}

function get_relayer_name() {
  docker ps --all --filter "name=rly-" --format "{{.Names}}"
}

# Use to preserve network state
function pause_containers() {
  local container_names=("$@")

  for container_name in "${container_names[@]}"; do
    echo "Pausing container $container_name..."
    docker pause "$container_name"
  done
}

function unpause_containers() {
  local container_names=("$@")

  for container_name in "${container_names[@]}"; do
    echo "Unpausing container $container_name..."
    docker unpause "$container_name"
  done
}

# Use to reset network state
function stop_and_remove_containers() {
  local container_names=("$@")

  for container_name in "${container_names[@]}"; do
    echo "Removing container $container_name..."
    if [ -z "$container_name" ]; then
      echo "Usage: remove_container <container_name>"
      return 1
    fi

    # Stop container if running
    docker stop "$container_name" > /dev/null

    # Get volumes
    volumes=$(docker inspect -f '{{range .Mounts}}{{.Name}} {{end}}' "$container_name")

    # Remove container
    docker rm -f "$container_name"

    # Remove volumes
    for volume in $volumes; do 
      docker volume rm "$volume" > /dev/null 
    done
    
    echo "Removed container $container_name and volumes: $volumes"
  done
}
