# Advance Stylus Benchmarks

This code repository aims to benchmark the gas costs between EVM and Stylus contracts for GpsStatementVerifier.verifyProofAndRegister functionality. The Solidity implementation can be found [here](https://github.com/starkware-libs/starkex-contracts/tree/master/evm-verifier/), but to be easier we have it in the repo (evm-verifier)


## Findings

To compare both implementation we will use this [transaction](https://dashboard.tenderly.co/tx/0x3acee509e2bb475eb7f35d60b439cd074e6af1a9db974136d0f2e78fd76ab90b?trace=0.1.1.5.0.431.17).
There are a few interesting moments we found out:
* Some of the contracts have been automatically generated, but the generator code is not publicly available
* The logic uses layout7 which does not exist in the `evm-verifier` repository. We found it from [here](https://github.com/Draply/Stark-verifier/tree/master/src/verifier/cpu/layout7).
* The logic uses poseidon arithmetic which does not exist in the `evm-verifier` repository. We found it from [here](https://github.com/Bisht13/post-quantum-eth-security/tree/main/contracts/periodic_columns).

> [!IMPORTANT]
> The provided numbers below are **L2_GAS** gas costs, because that's what's most important, since it represents the actual computational cost of the transactions, and not the `L1` calldata fees that are always fluctuating.


### MemoryPageFactRegistry (storage + calculations)

| Function  | Stylus Gas | Solidity Gas | Difference | % Difference |
|-----------|---------------|-----------------|-----------------|--------------|
| deploy | **2'919'340** | **864'926** | +2'054'414 | **237.5% more** |
| register_regular_memory_page | **1'145'908** | **675'308** | +470'600 | **69.6% more** |

**How to run**
> [!IMPORTANT] Need to have a running docker

```bash
/// 1. Run dev node
git clone https://github.com/OffchainLabs/nitro-devnode.git
cd nitro-devnode
bash ./run-dev-node.sh

/// 2. Deploy & Call Stylus 
make stylus-deploy-mpfr
/// 2.1 Open Makefile and change "contract" with the deployed contract address
make mpfr_register_mem_page
/// 2.2 Get the gas usage from the terminal

/// 3. Deploy & Call Solidity 
make solidity-deploy-mpfr
/// 3.1 Open Makefile and change "contract" with the deployed contract address
make mpfr_register_mem_page
/// 3.2 Get the gas usage from the terminal
```

### PoseidonPoseidonFullRoundKey0Column (pure calculations)

| Function  | Stylus Gas | Solidity Gas | Difference | % Difference |
|-----------|---------------|-----------------|-----------------|--------------|
| deploy | **1'860'416** | **216'778** | +1'643'638 | **758.2% more** |
| compute | **37'459** | **22'068** | +15'391 | **69.7% more** |

**How to run**
> [!IMPORTANT] Need to have a running docker

```bash
/// 1. Run dev node
git clone https://github.com/OffchainLabs/nitro-devnode.git
cd nitro-devnode
bash ./run-dev-node.sh

/// 2. Deploy & Call Stylus 
make stylus-deploy-poseidon-full-0
/// 2.1 Open Makefile and change "contract" with the deployed contract address
make ps_0_full_compute
/// 2.2 Get the gas usage from the terminal

/// 3. Deploy & Call Solidity 
make solidity-deploy-poseidon-full-0
/// 3.1 Open Makefile and change "contract" with the deployed contract address
make ps_0_full_compute
/// 3.2 Get the gas usage from the terminal
```
