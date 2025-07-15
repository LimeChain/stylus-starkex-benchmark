#! /bin/bash
RPC_URL_LOCAL="http://127.0.0.1:8547"
RPC_URL_MAINNET="https://mainnet.arbitrum.io/rpc"
RPC_URL=$RPC_URL_LOCAL
TIMEOUT=2
PK="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659"

# Mainnet addresses
fri_statement_verifier_address="0x30EfaAA99f8eFe310D9FdC83072e2a04c093d400"
merkle_statement_verifier_address="0x32a91Ff604AB2aDCd832e91D68b2f3f25358FdAd"

# Deployment order:
contracts=(
# - Auxiliary contracts:
  pedersen-hp-x-c
  pedersen-hp-y-c
  poseidon-frk-0-col
  poseidon-frk-1-col
  poseidon-frk-2-col
  poseidon-prk-0-col
  poseidon-prk-1-col
  verifier-init
# - Computation contracts:
  oods
  constraint-poly-preparer
  constraint-poly-finalizer
  constraint-poly
    fri-statement-verifier
# - Main contracts ZK logic contracts:
#   mpfr
  cpu-verifier
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
    local deploy_args=(--private-key="$PK" --endpoint="$RPC_URL")
    if [ "$name" == "cpu-verifier" ] \
    || [ "$name" == "verifier-init" ] \
    || [ "$name" == "fri-statement-verifier" ]; then
        deploy_args+=(--no-verify)
    fi
    DEPLOY_OUT=$(cargo stylus deploy "${deploy_args[@]}" 2>&1)
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

function deploy_contract_estimate_gas() {
    local name=$1
    local address
    pushd "stylus/$name"
    echo "Deploying '$name' contract from $(pwd)..."
    local var_name="${name//-/_}_address"
    DEPLOY_OUT=$(cargo stylus deploy  --estimate-gas --private-key=$PK --endpoint=$RPC_URL 2>&1)
    gas=$(echo "$DEPLOY_OUT" | grep -i 'deployment tx gas: ' )
    if [ -z "$gas" ]; then
        echo "❌ Deployment failed for '$name' contract"
        echo "---- Last 20 lines of output for debugging: ----"
        echo "$DEPLOY_OUT" | tail -20
        exit 1
    fi
    echo "✅ '$name': $gas"
    # eval "$var_name=$address"
    popd > /dev/null
}

# --- Deploy contracts
check_devnode
check_dependencies


# If local, deploy mock-provider and use it for the verifiers
if [ "$RPC_URL" == "$RPC_URL_LOCAL" ]; then
    deploy_contract "mock-provider"
    merkle_statement_verifier_address=$mock_provider_address
    fri_statement_verifier_address=$mock_provider_address
    echo "Using mock providers for the verifier"
    echo "✅ Merkle statement contract address: $merkle_statement_verifier_address"
    echo "✅ Fri statement contract address: $fri_statement_verifier_address"
fi


for name in "${contracts[@]}"; do
    deploy_contract "$name"
    var_name="${name//-/_}_address"
    eval "echo \"Captured address: \$$var_name\""
    if [ "$name" == "constraint-poly" ]; then
        echo "Setting addresses on $name via cast send..."
        CAST_OUT=$(cast send $constraint_poly_address "setAddresses(address,address)" \
            $constraint_poly_preparer_address $constraint_poly_finalizer_address \
            --rpc-url=$RPC_URL --private-key=$PK | grep "1 (success)")
        if [ -z "$CAST_OUT" ]; then
            echo "❌ Failed to call setAddresses on $constraint_poly_address"
            exit 1
        fi
        echo "✅ Successfully set addresses on $constraint_poly_address"
    elif [ "$name" == "fri-statement-verifier" ]; then
        echo "Setting addresses on $name via cast send..."
        CAST_OUT=$(cast send $fri_statement_verifier_address "init(address,address,address)" \
            $oods_address \
            $fri_statement_verifier_address \
            $merkle_statement_verifier_address \
            --rpc-url=$RPC_URL --private-key=$PK | grep "1 (success)")
        if [ -z "$CAST_OUT" ]; then
            echo "❌ Failed to call init on $fri_statement_verifier_address"
            exit 1
        fi
        echo "✅ Successfully set addresses on $fri_statement_verifier_address"
    elif [ "$name" == "cpu-verifier" ]; then
        echo "Setting addresses on $name via cast send..."
        CAST_OUT=$(cast send $cpu_verifier_address \
            "init(address,address,address,address,address,address,address,address,address,address)" \
            $constraint_poly_address \
            $pedersen_hp_x_c_address \
            $pedersen_hp_y_c_address \
            $poseidon_frk_0_col_address \
            $poseidon_frk_1_col_address \
            $poseidon_frk_2_col_address \
            $poseidon_prk_0_col_address \
            $poseidon_prk_1_col_address \
            $verifier_init_address \
            $fri_statement_verifier_address \
            --rpc-url=$RPC_URL --private-key=$PK | grep "1 (success)")
        if [ -z "$CAST_OUT" ]; then
            echo "❌ Failed to call init on $cpu_verifier_address"
            exit 1
        fi
        echo "✅ Successfully set addresses on $cpu_verifier_address"
    fi
done

# for name in "${contracts[@]}"; do
#     deploy_contract_estimate_gas "$name"
#     # var_name="${name//-/_}_address"
#     # eval "echo \"Captured address: \$$var_name\""
# done

