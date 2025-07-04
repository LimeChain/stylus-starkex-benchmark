# Advance Stylus Benchmarks

This code repository aims to benchmark the gas costs between EVM and Stylus contracts for GpsStatementVerifier.verifyProofAndRegister functionality. The Solidity implementation can be found [here](https://github.com/starkware-libs/starkex-contracts/tree/master/evm-verifier/), but to be easier we have it in the repo (evm-verifier)

## Setup

- Install [Docker].
- Install toolchain providing `cargo` using [rustup].
- Install the cargo stylus tool with `cargo install --force cargo-stylus`.

Note: If you are using Linux and encounter the `linker 'cc' not found` error during
stylus installation, make to sure to first install the necessary build tools with
the following command: `sudo apt-get install build-essential pkg-config libssl-dev -y`,
and retry installing the stylus tool.

[Docker]: https://docs.docker.com/engine/install/

[rustup]: https://rustup.rs/

## Findings

To compare both implementation we will use this [transaction](https://dashboard.tenderly.co/tx/0x3acee509e2bb475eb7f35d60b439cd074e6af1a9db974136d0f2e78fd76ab90b?trace=0.1.1.5.0.431.17).
There are a few interesting moments we found out:

- Some of the contracts have been automatically generated, but the generator code is not publicly available
- The logic uses layout7 which does not exist in the `evm-verifier` repository. We found it from [here](https://github.com/Draply/Stark-verifier/tree/master/src/verifier/cpu/layout7).
- The logic uses poseidon arithmetic which does not exist in the `evm-verifier` repository. We found it from [here](https://github.com/Bisht13/post-quantum-eth-security/tree/main/contracts/periodic_columns).

> [!IMPORTANT]
> The provided numbers below are **L2_GAS** gas costs, because that's what's most important, since it represents the actual computational cost of the transactions, and not the `L1` calldata fees that are always fluctuating.

| contract\gas cost | Deploy Stylus | Deploy Solidity | Deploy Diff | Call Stylus | Call Solidity | Call Diff |
|-----------|---------------|-----------------|-----------------|--------------|--------------|--------------|
| pedersen-hp-x-c | 5_393_222 | 4_230_105 | +1_163_117(27.5% more) | 151_995 | 32_712 | +119_283(364.6% more) |
| pedersen-hp-y-c | 5_381_591 | 4_231_137 | +1_150_454(27.2% more) | 152_012 | 32_712 | +119_300(364.6% more) |
| poseidon-frk-0-col | 1_869_383 | 166_753 | +1_702_630(10210% more) | 40_104 | 22_402 | +17_702(79.0% more) |
| poseidon-frk-1-col | 1_868_935 | 166_513 | +1_702_422(10213% more) | 40_104 | 22_402 | +17_702(79.0% more) |
| poseidon-frk-2-col | 1_869_104 | 166729 | +1_702_375(1021% more) | 40_115 | 22_402 | +17_713(79.1% more) |
| poseidon-prk-0-col | 2_311_839 | 618_006 | +1_693_833(274% more) | 52_644 | 23_589 | +29_055(123.2% more) |
| poseidon-prk-1-col | 2_045_185 | 360_456 | +1_684_729(467.4 more) | 45_534 | 22_907 | +22_627(107.5% more) |
| oods | 4_956_634 | 2_556_158 | +2_400_476(93.9% more) | 3_230_092 | 823_170 | +2_406_922(292.3% more) |
| constraint-poly-preparer | 5_171_185 | - | - | - | - | - |
| constraint-poly-finalizer | 4_436_374 | - | - | - | - | - |
| constraint-poly | 3_316_404 | 2_311_631 | 1_004_773(43.46% more) | 624_920 | 304_110 | +320_810(105.4% more) |
| mpfr | 4_296_904 | 665_328 | +3_631_576(545.8% more) | 1_151_619 | 675_308 | +476_311(70.5% more) |

## Specific cases

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
