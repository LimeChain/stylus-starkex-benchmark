# Advance Stylus Benchmarks

This code repository aims to benchmark the gas costs between EVM and Stylus contracts for GpsStatementVerifier.verifyProofAndRegister functionality. The Solidity implementation can be found [here](https://github.com/starkware-libs/starkex-contracts/tree/master/evm-verifier/), but to be easier we have it in the repo (evm-verifier)

## Setup

- Install [Docker].
- Install toolchain providing `cargo` using [rustup].
- Install the cargo stylus tool with `cargo install --force cargo-stylus`.
- Optionally: setup Virtual TestNets on Tenderly following the [Quickstart guide](https://docs.tenderly.co/virtual-testnets/quickstart).

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

## Contracts dependencies structure

```mermaid
graph TD
    %% Main entry point
    GPS[gps-sv<br/>Main GPS Statement Verifier] --> MPFR[mpfr<br/>Memory Page Fact Registry]
    GPS --> CPU[cpu-verifier<br/>CPU Verifier]
    
    %% CPU Verifier dependencies
    CPU --> CONSTRAINT[constraint-poly<br/>Constraint Polynomial]
    CPU --> PEDERSEN_X[pedersen-hp-x-c<br/>Pedersen Hash Points X]
    CPU --> PEDERSEN_Y[pedersen-hp-y-c<br/>Pedersen Hash Points Y]
    CPU --> POSEIDON_0[poseidon-frk-0-col<br/>Poseidon Full Round Key 0]
    CPU --> POSEIDON_1[poseidon-frk-1-col<br/>Poseidon Full Round Key 1]
    CPU --> POSEIDON_2[poseidon-frk-2-col<br/>Poseidon Full Round Key 2]
    CPU --> POSEIDON_P0[poseidon-prk-0-col<br/>Poseidon Partial Round Key 0]
    CPU --> POSEIDON_P1[poseidon-prk-1-col<br/>Poseidon Partial Round Key 1]
    CPU --> INIT[verifier-init<br/>Verifier Initialization]
    CPU --> FRI[fri-statement-verifier<br/>FRI Statement Verifier]
    
    %% Constraint Polynomial dependencies
    CONSTRAINT --> PREP[constraint-poly-preparer<br/>Constraint Poly Preparer]
    CONSTRAINT --> FIN[constraint-poly-finalizer<br/>Constraint Poly Finalizer]
    
    %% FRI Statement Verifier dependencies
    FRI --> FriStatementVerifier
    FRI --> MerkleStatementVerifier
    FRI --> OODS[oods<br/>Out of Domain Sampling]
    
    %% Styling
    classDef mainContract fill:#1976d2,stroke:#0d47a1,stroke-width:4px,color:#ffffff
    classDef computeContract fill:#7b1fa2,stroke:#4a148c,stroke-width:3px,color:#ffffff
    classDef auxContract fill:#388e3c,stroke:#1b5e20,stroke-width:3px,color:#ffffff
    classDef mockContract fill:#f57c00,stroke:#e65100,stroke-width:3px,color:#ffffff
    
    class GPS mainContract
    class CPU,MPFR computeContract
    class CONSTRAINT,OODS,FRI computeContract
    class PREP,FIN computeContract
    class PEDERSEN_X,PEDERSEN_Y,POSEIDON_0,POSEIDON_1,POSEIDON_2,POSEIDON_P0,POSEIDON_P1,INIT,MerkleStatementVerifier,FriStatementVerifier auxContract
    class MOCK_FRI,MOCK_MERKLE mockContract
```

## Gas costs
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
| cpu-verifier | 5_407_239 | 4_370_831 | +1_036_408(23,7% more) | 470_382 | 935_831 | -465_449(49% less) |
| gps(full flow) | 4_286_090 | 2_137_930  | +2_148_160(100.5%% more) | 7_505_490 | 4_523_567 | +2_981_923(65% more) |

Original gas costs were taken from the [Transaction Trace](https://app.sentio.xyz/tx/1/0x3f4e2a13b6c2356ad7f2c2af62e2eb0fb7bee626a563ccb49a8f73c684bd6eef/debug?trace=145364).

## How to run
### Deploy
> [!IMPORTANT] Need to have a running docker

```bash
/// 1. Run dev node
git clone https://github.com/OffchainLabs/nitro-devnode.git
cd nitro-devnode
bash ./run-dev-node.sh

/// 2. Run full flow Deployment
make deploy
```
### Gps Full flow
> [!IMPORTANT] Extract contract address from the deployment step and use it in the next steps
```bash
/// 3. Open Makefile and change 'gps_contract' value with the deployed contract address
/// 4. Run full flow Call
make gps
/// 5. Get the gas usage from the terminal
```

### CPU Verifier
> [!IMPORTANT] Extract CPU Verifier contract address from the deployment step and use it in
```bash
/// 3.1 Go to test directory and make test.sh executable
cd test/
chmod +x test.sh
/// 3.2 Change "cpu_contract" in test.sh file with the deployed contract address
/// 3.3 Run the test.sh script
./test.sh
/// 3.4 Expect the output to be "----------CORRECT----------"
```

### Poseidon AUX contracts
> [!IMPORTANT] Extract Poseidon contracts addresses from the deployment step and use it in the next steps

```bash
/// 3.1 Open Makefile and change "poseidon_contract" with the deployed contract address
make poseidon
/// 3.1 Go to test directory
cd test/
/// 3.2 Put the CPU Verifier contract address in the test file

### Poseidon AUX contracts
> [!IMPORTANT] Extract Poseidon contracts addresses from the deployment step and use it in the next steps

```bash
/// 3.1 Open Makefile and change "poseidon_contract" with the deployed contract address
make poseidon
/// 3.2 Get the gas usage from the terminal
```

### Pedersen AUX contracts
> [!IMPORTANT] Extract Pedersen contracts addresses from the deployment step and use it in the
next steps

```bash
/// 3.1 Open Makefile and change "pedersen_contract" with the deployed contract address
make pedersen
/// 3.2 Get the gas usage from the terminal
```

### Oods (Out of Domain Sampling)
> [!IMPORTANT] Extract Oods contract address from the deployment step and use it in the
next steps

```bash
/// 3.1 Open Makefile and change "oods_contract" with the deployed contract address
make oods
/// 3.2 Get the gas usage from the terminal
```

### Constraint Polynomial
> [!IMPORTANT] Extract Constraint Polynomial contract address from the deployment step and use it in the next steps

```bash
/// 3.1 Open Makefile and change "poly_contract" with the deployed contract address
make constraint_poly_full
/// 3.2 Get the gas usage from the terminal
make constraint_poly_full_estimate
```

### MemoryPageFactRegistry (storage + calculations)
> [!IMPORTANT]  Extract contract address from the deployment step and use it in the next steps

```bash
/// 3.1 Open Makefile and change "contract" with the deployed contract address
make mpfr_register_mem_page
/// 3.2 Get the gas usage from the terminal
```

## Unit tests
Every contract from ./stylus has its own unit tests, which can be run with
```bash
/// e.g. for pedersen-hp-x-c contract
cd ./stylus/pedersen-hp-x-c
cargo test --release
```