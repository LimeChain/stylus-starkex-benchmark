pk=0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659

MPFR_INPUT := $(shell tr '\n' ' ' < ./inputs/mpfr_from_gps.txt)

rpc_url=http://localhost:8547
contract=0xC2C0c3398915A2d2E9C33C186AbFEF3192Ee25E8

# -------------------------------------------------------------------------------------------------
# NITRO
# -------------------------------------------------------------------------------------------------
## MemoryPageFactRegistry
# Gas used: 665328
.PHONY: solidity-deploy-mpfr
solidity-deploy-mpfr:
	forge script script/DeployMemoryPageFactRegistry.s.sol \
	--rpc-url ${rpc_url} --private-key $(pk) \
	--broadcast --chain-id 412346

.PHONY: stylus-deploy-mpfr
stylus-deploy-mpfr:
	cd ./stylus/mpfr && cargo stylus deploy --private-key=$(pk)

.PHONY: nitro-test-mpfr
nitro-test-mpfr:
	forge test --match-contract MemoryPageFactRegistryTest \
	--fork-url nitro -vvv

.PHONY: mpfr_register_mem_page
mpfr_register_mem_page:
	@cast send $(contract) "registerRegularMemoryPage(uint256[],uint256,uint256,uint256)" \
	$(MPFR_INPUT) \
	1923983994410949646266215635478491917832882166179969396251746181413976269170 \
	2548115266380774413420845979236209449237376742700778263417656557146680537758 \
	3618502788666131213697322783095070105623107215331596699973092056135872020481 \
	--rpc-url $(rpc_url) --private-key $(pk) -vvv --gas-limit 2000000


.PHONY: pederson_x_deploy
pederson_x_deploy:
	forge script script/PedersenHashPointsXColumn.s.sol:PedersenHashPointsXColumnDeploy \
	--rpc-url ${rpc_url} \
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
	--rpc-url ${rpc_url} \
	--private-key $(pk) \
	--broadcast

.PHONY: cpu_constraint_poly_deploy
cpu_constraint_poly_deploy:
	forge script script/CpuConstraintPoly.s.sol:CpuConstraintPolyScript \
	--rpc-url anvil \
	--private-key $(nitro_pk) \
	--broadcast

.PHONY: periodic_columns_pedersen_test
periodic_columns_pedersen_test:
	forge test --match-test testPedersenHashPointsColumnCompute \
	--fork-url ${rpc_url}  -vvv --gas-report

.PHONY: periodic_columns_poseidon_test
periodic_columns_poseidon_test:
	forge test --match-test testPoseidonPoseidonColumnCompute \
	--fork-url nitro  -vvv

.PHONY: periodic_columns_pederson_frk_0_col_test
periodic_columns_pederson_frk_0_col_test:
	forge test --match-test testPedersonFrk0ColumnCompute \
	--fork-url ${rpc_url}  -vvv 

.PHONY: cpu_constraint_poly_test
cpu_constraint_poly_test:
	forge test --match-test testCpuConstraintPolyFromRealTx \
	--fork-url anvil  -vvv --gas-limit 2000000000

per_col_contract=0x72219e4c1b76276253a852ab058374d1dd5529be
# cumulativeGasUsed    23252
.PHONY: poseidonPartialRoundKey0ColumnCast
poseidonPartialRoundKey0ColumnCast:
	cast send $(contract) "compute(uint256)" \
	513761785516736576210258345954495650460389361631034617172115002511570125974 \
	--rpc-url ${rpc_url} --private-key $(pk) -vvv --gas-limit 2000000

# frk0_contract=0x4A2bA922052bA54e29c5417bC979Daaf7D5Fe4f4
frk0_contract=0x47cec0749bd110bc11f9577a70061202b1b6c034
# cumulativeGasUsed
# Solidity:    22068
# Stylus: 37459
# -- Partial round key 0 column
# cumulativeGasUsed
# Solidity: 23252
# Stylus: 50564
# PoseidonPoseidonPartialRoundKey0Column
# Stylus: 50294
.PHONY: poseidonPoseidonFullRoundKey0ColumnCast
poseidonPoseidonFullRoundKey0ColumnCast:
	cast send $(frk0_contract) "compute(uint256)" \
	513761785516736576210258345954495650460389361631034617172115002511570125974 \
	--rpc-url ${rpc_url} --private-key $(pk) -vvv --gas-limit 2000000


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


poly_contract=0x525c2aba45f66987217323e8a05ea400c65d06dc
fin_contract=0x525c2aba45f66987217323e8a05ea400c65d06dc
preparer_contract=0x4a2ba922052ba54e29c5417bc979daaf7d5fe4f4
.PHONY: constraint_poly_cast
constraint_poly_cast:
	cast call 0x525c2aba45f66987217323e8a05ea400c65d06dc $$(cat poly_input)  --rpc-url nitro

.PHONY: constraint_poly_prep
constraint_poly_prep:
	@cast call $(preparer_contract) $$(cat stylus/testdata/poly_input.hex)  --rpc-url nitro

.PHONY: constraint_poly_fin
constraint_poly_fin:
	cast call 0x525c2aba45f66987217323e8a05ea400c65d06dc $$(cat stylus/testdata/fin_input.hex)  --rpc-url nitro

.PHONY: constraint_poly_fin-test
constraint_poly_fin-test:
	@EXPECTED="0x06830dfba344bbbb4521412ab453a5883b76d7649286a365017d2eb2984ad636"; \
	ACTUAL=$$(cast call 0x525c2aba45f66987217323e8a05ea400c65d06dc $$(cat stylus/testdata/fin_input.hex) --rpc-url nitro); \
	if [ "$$ACTUAL" = "$$EXPECTED" ]; then \
		echo "Test passed!"; \
		echo "$$ACTUAL"; \
	else \
		echo "Test failed: expected $$EXPECTED, got $$ACTUAL"; \
		exit 1; \
	fi
