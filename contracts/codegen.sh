# a script to generate codegen files
# usage example: ./codegen.sh adapter-dex-stargaze

cd "./contracts/$1"

DIR_NAME=$(echo ${PWD##*/})
CODEGEN_PATH="./codegen"
INTERFACES_PATH="../../scripts/src/common/codegen"

# generate schema
cargo schema

# fix for ts-codegen MissingPointerError
# https://github.com/CosmWasm/ts-codegen/issues/90
rm -rf ./schema/raw

mkdir -p $CODEGEN_PATH

cosmwasm-ts-codegen generate \
  --plugin client \
	--plugin message-composer \
  --schema ./schema \
  --out $CODEGEN_PATH \
  --name $DIR_NAME \
  --no-bundle

cp -r "$CODEGEN_PATH/." $INTERFACES_PATH
rm -rf $CODEGEN_PATH
