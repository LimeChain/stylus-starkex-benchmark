

pk=0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659
nitro_pk = 0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659
anvil_pk = 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
inputpath = /Users/sergei.milev/wrksps/starkex/inputs/mpfr_from_gps.txt

MPFR_INPUT := $(shell tr '\n' ' ' < /Users/sergei.milev/wrksps/starkex/inputs/mpfr_from_gps.txt)

# Contract addresses on Anvil
ca_anvil_mpfr=0xb69FC79100eDd058f9c96c0a13C80124aC1a7D77
ca_nitro_mpfr = 0x525c2aBA45F66987217323E8a05EA400C65D06DC

.PHONY: deploy-wasm
deploy-wasm:
	cargo stylus deploy --endpoint='http://localhost:8547' --private-key=$(pk)

# ANVIL
## MemoryPageFactRegistry 
# Gas used: 665328
.PHONY: anvil-deploy-mpfr
anvil-deploy-mpfr:
	forge script script/DeployMemoryPageFactRegistry.s.sol \
	--rpc-url tenderly_fork \
	--private-key $(anvil_pk) \
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
	cast send 0x11729c8891536e1b2143b81db3e8d2238eee6b1b "registerRegularMemoryPage(uint256[],uint256,uint256,uint256)" "[1,100,2,200]" 5 3 57896044618658097711785492504343953926634992332820282019728792003956564819969 \
	--rpc-url arb_fork --private-key $(nitro_pk) -vvv

.PHONY: nitro-cast-mpfr-registerRegularMemoryPage-big-input
nitro-cast-mpfr-registerRegularMemoryPage-big-input:
	@cast send 0x11729c8891536e1b2143b81db3e8d2238eee6b1b "registerRegularMemoryPage(uint256[],uint256,uint256,uint256)" \
	$(MPFR_INPUT) \
	1923983994410949646266215635478491917832882166179969396251746181413976269170 \
	2548115266380774413420845979236209449237376742700778263417656557146680537758 \
	3618502788666131213697322783095070105623107215331596699973092056135872020481 \
	--rpc-url arb_fork --private-key $(nitro_pk) -vvv --gas-limit 3000000000