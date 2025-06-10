

pk=0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659
nitro_pk = 0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659
anvil_pk = 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
inputpath = /Users/sergei.milev/wrksps/starkex/inputs/mpfr_from_gps.txt

MPFR_INPUT := $(shell tr '\n' ' ' < /Users/sergei.milev/wrksps/starkex/inputs/mpfr_from_gps.txt)

# Contract addresses on Anvil
ca_anvil_mpfr=0x4Af567288e68caD4aA93A272fe6139Ca53859C70
ca_nitro_mpfr = 0x4af567288e68cad4aa93a272fe6139ca53859c70
ca_td_mpfr=0x525c2aBA45F66987217323E8a05EA400C65D06DC

rpc_url=http://localhost:8547


contract=0x525c2aba45f66987217323e8a05ea400c65d06dc

.PHONY: deploy-wasm
deploy-wasm:
	cargo stylus deploy --endpoint=$(rpc_url) --private-key=$(pk)

# ANVIL
## MemoryPageFactRegistry 
# Gas used: 665328
.PHONY: anvil-deploy-mpfr
anvil-deploy-mpfr:
	forge script script/DeployMemoryPageFactRegistry.s.sol \
	--rpc-url $(rpc_url) \
	--private-key $(pk) \
	--broadcast

.PHONY: anvil-test-mpfr
anvil-test-mpfr:
	forge test --match-contract MemoryPageFactRegistryTest \
	--fork-url anvil -vvv

# gasUsed              
# 70307, 30240, 30240
.PHONY: anvil-cast-mpfr-registerRegularMemoryPage
anvil-cast-mpfr-registerRegularMemoryPage:
	cast send $(ca_anvil_mpfr) "registerRegularMemoryPage(uint256[],uint256,uint256,uint256)" \
	$(MPFR_INPUT) \
	1923983994410949646266215635478491917832882166179969396251746181413976269170 \
	2548115266380774413420845979236209449237376742700778263417656557146680537758 \
	3618502788666131213697322783095070105623107215331596699973092056135872020481 \
	--rpc-url anvil --private-key $(anvil_pk) -vvv

.PHONY: anvil-cast-mpfr-registerRegularMemoryPage-big-input
anvil-cast-mpfr-registerRegularMemoryPage-big-input:
	cast send $(ca_anvil_mpfr) "registerRegularMemoryPage(uint256[],uint256,uint256,uint256)" $$(cat $inputpath)\
	--rpc-url anvil --private-key $(anvil_pk) -vvv

.PHONY: tenderly-fork-cast
tenderly-fork-cast:
	cast send 0xc91B651f770ed996a223a16dA9CCD6f7Df56C987 "registerRegularMemoryPage(uint256[],uint256,uint256,uint256)" "[1,100,2,200]" 5 3 57896044618658097711785492504343953926634992332820282019728792003956564819969 \
	--rpc-url tenderly_fork --private-key $(anvil_pk) -vvv

# -------------------------------------------------------------------------------------------------
# NITRO
# -------------------------------------------------------------------------------------------------
## MemoryPageFactRegistry
# Gas used: 665328
.PHONY: nitro-deploy-mpfr
nitro-deploy-mpfr:
	forge script script/DeployMemoryPageFactRegistry.s.sol \
	--rpc-url nitro --private-key $(nitro_pk) \
	--broadcast --chain-id 412346

.PHONY: nitro-test-mpfr
nitro-test-mpfr:
	forge test --match-contract MemoryPageFactRegistryTest \
	--fork-url nitro -vvv

# gasUsed
# 70307, 30240, 30240
# WASM implementation
# gasUsed
# 85360, 45342, 45342
.PHONY: nitro-cast-mpfr-registerRegularMemoryPage
nitro-cast-mpfr-registerRegularMemoryPage:
	cast send $(ca_td_mpfr) "registerRegularMemoryPage(uint256[],uint256,uint256,uint256)" "[1,100,2,200]" 5 3 57896044618658097711785492504343953926634992332820282019728792003956564819969 \
	--rpc-url tenderly_fork --private-key $(nitro_pk) --gas-limit 2000000 -vvv

.PHONY: nitro-cast-mpfr-registerRegularMemoryPage-big-input
nitro-cast-mpfr-registerRegularMemoryPage-big-input:
	@cast send $(contract) "registerRegularMemoryPage(uint256[],uint256,uint256,uint256)" \
	$(MPFR_INPUT) \
	1923983994410949646266215635478491917832882166179969396251746181413976269170 \
	2548115266380774413420845979236209449237376742700778263417656557146680537758 \
	3618502788666131213697322783095070105623107215331596699973092056135872020481 \
	--rpc-url $(rpc_url) --private-key $(pk) -vvv --gas-limit 2000000


.PHONY: pederson_x_deploy
pederson_x_deploy:
	forge script script/PedersenHashPointsXColumn.s.sol:PedersenHashPointsXColumnDeploy \
	--rpc-url nitro \
	--private-key $(nitro_pk) \
	--broadcast

# == Logs ==
#   PedersenHashPointsXColumn deployed at: 0x525c2aBA45F66987217323E8a05EA400C65D06DC - 32712
#   PedersenHashPointsYColumn deployed at: 0x85D9a8a4bd77b9b5559c1B7FCb8eC9635922Ed49
#   PoseidonPoseidonFullRoundKey0Column deployed at: 0x4A2bA922052bA54e29c5417bC979Daaf7D5Fe4f4 - 22068
#   PoseidonPoseidonFullRoundKey1Column deployed at: 0x4Af567288e68caD4aA93A272fe6139Ca53859C70
#   PoseidonPoseidonFullRoundKey2Column deployed at: 0x3DF948c956e14175f43670407d5796b95Bb219D8
#   PoseidonPoseidonPartialRoundKey0Column deployed at: 0x75E0E92A79880Bd81A69F72983D03c75e2B33dC8 - 23252
#   PoseidonPoseidonPartialRoundKey1Column deployed at: 0xF5FfD11A55AFD39377411Ab9856474D2a7Cb697e
.PHONY: periodic_columns_deploy
periodic_columns_deploy:
	forge script script/PeriodicColumns.s.sol:PeriodicColumns \
	--rpc-url nitro \
	--private-key $(nitro_pk) \
	--broadcast

.PHONY: periodic_columns_pedersen_test
periodic_columns_pedersen_test:
	forge test --match-test testPedersenHashPointsColumnCompute \
	--fork-url nitro  -vvv --gas-report

.PHONY: periodic_columns_poseidon_test
periodic_columns_poseidon_test:
	forge test --match-test testPoseidonPoseidonColumnCompute \
	--fork-url nitro  -vvv --gas-report

.PHONY: periodic_columns_pederson_frk_0_col_test
periodic_columns_pederson_frk_0_col_test:
	forge test --match-test testPedersonFrk0ColumnCompute \
	--fork-url nitro  -vvv 

per_col_contract=0x75E0E92A79880Bd81A69F72983D03c75e2B33dC8
# cumulativeGasUsed    23252
.PHONY: poseidonPartialRoundKey0ColumnCast
poseidonPartialRoundKey0ColumnCast:
	cast send $(per_col_contract) "compute(uint256)" \
	513761785516736576210258345954495650460389361631034617172115002511570125974 \
	--rpc-url nitro --private-key $(nitro_pk) -vvv --gas-limit 2000000

# frk0_contract=0x4A2bA922052bA54e29c5417bC979Daaf7D5Fe4f4
frk0_contract=0x47cec0749bd110bc11f9577a70061202b1b6c034
# cumulativeGasUsed
# Solidity:    22068
# Stylus: 37459
# -- Partial round key 0 column
# cumulativeGasUsed
# Solidity: 23252
# Stylus: 50564
.PHONY: poseidonPoseidonFullRoundKey0ColumnCast
poseidonPoseidonFullRoundKey0ColumnCast:
	cast send $(frk0_contract) "compute(uint256)" \
	513761785516736576210258345954495650460389361631034617172115002511570125974 \
	--rpc-url nitro --private-key $(nitro_pk) -vvv --gas-limit 2000000


pedersen_contract=0x8e1308925a26cb5cF400afb402d67B3523473379

# cumulativeGasUsed    
# PedersenHashPointsXColumn
# Solidity: 32712
# Stylus: 151995
# PedersenHashPointsYColumn
# Solidity: 32712
# Stylus: 152012
.PHONY: pedersen_cast
pedersen_cast:
	cast send $(pedersen_contract) "compute(uint256)" \
	2502371038239847331946845555940821891939660827069539886818086403686260021246 \
	--rpc-url nitro --private-key $(nitro_pk) -vvv --gas-limit 2000000




.PHONY: pedersen_cast_call
pedersen_cast_call:
	cast call $(pedersen_contract) "compute(uint256)" \
	2502371038239847331946845555940821891939660827069539886818086403686260021246 \
	--rpc-url nitro  --private-key $(nitro_pk) -vvv