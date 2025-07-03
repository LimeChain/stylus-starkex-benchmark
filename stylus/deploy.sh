#! /bin/bash
# Deployment order:
# - Auxiliary contracts:
#  - pedersen-hp-x-c
#  - pedersen-hp-y-c
#  - poseidon-frk-0-col
#  - poseidon-frk-1-col
#  - poseidon-frk-2-col
#  - poseidon-prk-0-col
#  - poseidon-prk-1-col
# - Computation contracts:
#  - oods
#  - constraint-poly-preparer
#  - constraint-poly-finalizer
#  - constraint-poly
# - Main contracts ZK logic contracts:
#  - memory-page-fact-registry
#  - cpu
#  - gps
# set -euo pipefail

RPC_URL="http://127.0.0.1:8547"
TIMEOUT=2
PK="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659"
contracts=(
  pedersen-hp-x-c
  pedersen-hp-y-c
  poseidon-frk-0-col
  poseidon-frk-1-col
  poseidon-frk-2-col
  poseidon-prk-0-col
  poseidon-prk-1-col
  oods
  constraint-poly-preparer
  constraint-poly-finalizer
  constraint-poly
  mpfr
#   cpu
#   gps
)

function check_devnode() {
  echo "Checking if devnode is running on $RPC_URL ..."
  response=$(curl -s -X POST --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' -H "Content-Type: application/json" $RPC_URL)
  if [[ $response == *"result"* ]]; then
    echo "✅ Devnode is UP on $RPC_URL! Response: $response"
    return 0
  else
    echo "❌ Devnode is NOT running or not responding on $RPC_URL"
    exit 1
  fi
}

function check_dependencies() {
    echo "Checking Rust dependencies..."
    cargo_st_v=$(cargo stylus --version)
    if [ $? -ne 0 ]; then
        echo "❌ Cargo stylus is not installed"
        exit 1
    fi
    wasm_target=$(rustup target list | grep "wasm32-unknown-unknown (installed)")
    if [ -z "$wasm_target" ]; then
        echo "❌ Wasm target is not installed"
        exit 1
    fi
    cast_v=$(cast --version)
    if [ $? -ne 0 ]; then
        echo "❌ Cast is not installed"
        exit 1
    fi
    echo "✅ Cargo stylus version: $cargo_st_v"
    echo "✅ Wasm target: $wasm_target"
    echo "✅ Cast version: $(echo $cast_v | cut -d' ' -f3)"
    echo "-------------------------------------------------"
}

function deploy_contract() {
    local name=$1
    local address
    pushd "stylus/$name"
    echo "Deploying '$name' contract from $(pwd)..."
    local var_name="${name//-/_}_address"
    DEPLOY_OUT=$(cargo stylus deploy  --private-key=$PK --endpoint=$RPC_URL 2>&1)
    address=$(echo "$DEPLOY_OUT" | grep -i 'deployed code at address' | grep -Eo '0x[a-fA-F0-9]{40}')
    if [ -z "$address" ]; then
        echo "❌ Deployment failed for '$name' contract"
        echo "---- Last 20 lines of output for debugging: ----"
        echo "$DEPLOY_OUT" | tail -20
        exit 1
    fi
    echo "✅ '$name' contract at $address"
    eval "$var_name=$address"
    popd > /dev/null
}

# --- Deploy contracts
check_devnode
check_dependencies

for name in "${contracts[@]}"; do
    deploy_contract "$name"
    var_name="${name//-/_}_address"
    eval "echo \"Captured address: \$$var_name\""
done

# --- SetUp contracts
echo "Setting addresses on constraint-poly via cast send..."
CAST_OUT=$(cast send $constraint_poly_address "setAddresses(address,address)" \
    $constraint_poly_preparer_address $constraint_poly_finalizer_address \
    --rpc-url=$RPC_URL --private-key=$PK | grep "1 (success)")

if [ -z "$CAST_OUT" ]; then
    echo "❌ Failed to call setAddresses on $constraint_poly_address"
    exit 1
fi
echo "✅ Successfully set addresses on $constraint_poly_address"
