pk=0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659

MPFR_INPUT := $(shell tr '\n' ' ' < ./inputs/mpfr_from_gps.txt)

# GPS inputs
GPS_INPUT_PROOF_PARAMS := $(shell tr '\n' ' ' < ./inputs/gps_input_proof_params.txt)
GPS_INPUT_PROOF := $(shell tr '\n' ' ' < ./inputs/gps_input_proof.txt)
GPS_INPUT_TASK := $(shell tr '\n' ' ' < ./inputs/gps_input_task.txt)
GPS_INPUT_AUX := $(shell tr '\n' ' ' < ./inputs/gps_input_aux.txt)

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

.PHONY: stylus-deploy-cpu-verifier
stylus-deploy-cpu-verifier:
	cd ./stylus/ && cargo stylus deploy --private-key=$(pk)

.PHONY: nitro-test-mpfr
nitro-test-mpfr:
	forge test --match-contract MemoryPageFactRegistryTest \
	--fork-url nitro -vvv

mpfr_contract=0x24d64cefe06627ebd605b050e7a8dec756f65547
.PHONY: mpfr_register_mem_page
mpfr_register_mem_page:
	@cast call $(mpfr_contract) "registerRegularMemoryPage(uint256[],uint256,uint256,uint256)" \
	$(MPFR_INPUT) \
	1889307229587391548520309518094765161966447786756161259520600117314548314282 \
	128717201870591596518410513636207059091228026160841752656809259718526660533 \
	3618502788666131213697322783095070105623107215331596699973092056135872020481 \
	--rpc-url $(rpc_url) --private-key $(pk) -vvv --gas-limit 2000000

gps_contract=0x05d25474a141659baa4497afa7fa4a6932632a5a
.PHONY: gps
gps:
	cast send $(gps_contract) "verifyProofAndRegister(uint256[], uint256[], uint256[], uint256[], uint256)" \
	$(GPS_INPUT_PROOF_PARAMS) \
	$(GPS_INPUT_PROOF) \
	$(GPS_INPUT_TASK) \
	$(GPS_INPUT_AUX) \
	0 \
	--rpc-url $(rpc_url) \
	--private-key $(pk) -vvv --gas-limit 200000000000000000


.PHONY: gps_estimate
gps_estimate:
	cast estimate $(gps_contract) "verifyProofAndRegister(uint256[], uint256[], uint256[], uint256[], uint256)" \
	$(GPS_INPUT_PROOF_PARAMS) \
	$(GPS_INPUT_PROOF) \
	$(GPS_INPUT_TASK) \
	$(GPS_INPUT_AUX) \
	0 \
	--rpc-url $(rpc_url) \
	--private-key $(pk) -vvv --gas-limit 200000000000000000

.PHONY: pederson_x_deploy
pederson_x_deploy:
	forge script script/PedersenHashPointsXColumn.s.sol:PedersenHashPointsXColumnDeploy \
	--rpc-url ${rpc_url} \
	--private-key $(nitro_pk) \
	--broadcast

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
.PHONY: poseidonPartialRoundKey0ColumnCast
poseidonPartialRoundKey0ColumnCast:
	cast send $(contract) "compute(uint256)" \
	513761785516736576210258345954495650460389361631034617172115002511570125974 \
	--rpc-url ${rpc_url} --private-key $(pk) -vvv --gas-limit 2000000

poseidon_contract=0x47cec0749bd110bc11f9577a70061202b1b6c034
.PHONY: poseidon
poseidon:
	cast send $(poseidon_contract) "compute(uint256)" \
	513761785516736576210258345954495650460389361631034617172115002511570125974 \
	--rpc-url ${rpc_url} --private-key $(pk) -vvv --gas-limit 2000000


pedersen_contract=0x8e1308925a26cb5cF400afb402d67B3523473379
.PHONY: pedersen
pedersen:
	cast send $(pedersen_contract) "compute(uint256)" \
	2502371038239847331946845555940821891939660827069539886818086403686260021246 \
	--rpc-url nitro --private-key $(nitro_pk) -vvv --gas-limit 2000000


.PHONY: pedersen_cast_call
pedersen_cast_call:
	cast call $(pedersen_contract) "compute(uint256)" \
	2502371038239847331946845555940821891939660827069539886818086403686260021246 \
	--rpc-url nitro  --private-key $(nitro_pk) -vvv


poly_contract=0x05C98569CA566a2035b87dE7d1b623C950798035
fin_contract=0x525c2aba45f66987217323e8a05ea400c65d06dc
preparer_contract=0x4a2ba922052ba54e29c5417bc979daaf7d5fe4f4
.PHONY: constraint_poly_full
constraint_poly_full:
	cast call $(poly_contract) $$(cat stylus/testdata/poly_input.hex)  --rpc-url nitro

.PHONY: constraint_poly_full_estimate
constraint_poly_full_estimate:
	cast estimate $(poly_contract) $$(cat stylus/testdata/poly_input.hex) --rpc-url nitro --private-key $(pk) -vvv --gas-limit 2000000

.PHONY: constraint_poly_setup
constraint_poly_setup:
	cast send $(preparer_contract) $$(cat stylus/testdata/poly_input.hex)  --rpc-url nitro

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

poly_contract=0xfffb0ed9d6538e4b01cc0291814eaa4f2cc58254
.PHONY: init_poly_contract
init_poly_contract:
	@cast send $(poly_contract) "setAddresses(address,address)" \
	0xe2b648b8e4bfa271f327e6c6c71fbbdf5a2395d4 \
	0xa11df8dbba014facc83cdeb9b4b6a98796254007 \
	--rpc-url $(rpc_url) --private-key $(pk) -vvv --gas-limit 2000000

fri_contract=0xacd8c4dc161bef1cde93c14861589b35f5000a19
.PHONY: init_fri_contract
init_fri_contract:
	@cast send $(fri_contract) "init(address,address,address)" \
	0x6d27fb544ddd3647443e997a21f0fe1dfcde3057 \
	0x0000000000000000000000000000000000000000 \
	0x0000000000000000000000000000000000000000 \
	--rpc-url $(rpc_url) --private-key $(pk) -vvv --gas-limit 2000000

cpu_contract=0xd01207dd6eb9359f7572f658de0cb4ec98858da5
.PHONY: init_cpu_contract
init_cpu_contract:
	@cast send $(cpu_contract) "init(address,address,address,address,address,address,address,address,address,address)" \
	0xfffb0ed9d6538e4b01cc0291814eaa4f2cc58254 \
    0xd48eb52a301a3f72c81ab126056cf204b3bd2b0c \
    0x6ba2c7e189daebe5b596d5b76b4d43f7b38d9de5 \
    0x3b5b80304dfda6ba079161acfad648959c8745dd \
    0x32c0013bdbbe645a85dd8bcc431d1b672bf0cfa1 \
    0xa6788c256e8a1d2470df159a8b74ab86507aac10 \
    0x801abd1cb75fefd0057943ba99ee83775c522831 \
    0x07e2a25d805edf05f449d35fd1c846e8b1b4a140 \
    0xfb493c75b7c2e2dca54f1c0f53ecf057b1de4e4a \
    0xacd8c4dc161bef1cde93c14861589b35f5000a19 \
	--rpc-url $(rpc_url) --private-key $(pk) -vvv

PROOF_PARAMS := $(shell tr '\n' ' ' < ./inputs/proof_params.txt)
PROOF := $(shell tr '\n' ' ' < ./inputs/proof.txt)
PUBLIC_INPUT := $(shell tr '\n' ' ' < ./inputs/public_input.txt)

.PHONY: verify_proof
verify_proof:
	cast call $(cpu_contract) "verifyProofExternal(uint256[],uint256[],uint256[])" \
	$(PROOF_PARAMS) \
	$(PROOF) \
	$(PUBLIC_INPUT) \
	--rpc-url $(rpc_url) --private-key $(pk) -vvv --gas-limit 20000000


.PHONY: test
test:
	cd ./test && bash ./test.sh

oods_contract=0x4a2ba922052ba54e29c5417bc979daaf7d5fe4f4
.PHONY: oods
oods:
	@cast send -vvv $(oods_contract) $$(cat stylus/testdata/oods_input.hex ) --rpc-url nitro --private-key $(pk) --gas-limit 2000000

.PHONY: deploy
deploy:
	stylus/deploy.sh
