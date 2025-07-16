//!
//! GPS Statement Verifier
//! The following contract implements the GPS statement verifier example from Foundry.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]
extern crate alloc;

mod bootloader;
mod consts;
use alloc::{vec, vec::Vec};
use bootloader::{
    BootloaderCompiledProgram, APPLICATION_BOOTLOADER_PROGRAM_HASH, SIMPLE_BOOTLOADER_PROGRAM_HASH,
};
use consts::{page_info::*, public_input_offsets};

/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{
    alloy_primitives::{uint, Address, FixedBytes, I256, U256},
    call::{self, Call, MethodError},
    // console,
    crypto::keccak,
    prelude::*,
    storage::*,
};

macro_rules! require {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            return Err($msg.as_bytes().to_vec());
        }
    };
}

sol_interface! {
    interface IMemoryPageFactRegistry {
        function registerRegularMemoryPage(uint256[] memory memory_pairs, uint256 z, uint256 alpha, uint256 prime) external returns (bytes32, bytes32, uint256);
    }
    interface ICairoVerifierContract {
        function verifyProofExternal(uint256[] calldata proofParams, uint256[] calldata proof, uint256[] calldata publicInput) external returns (uint256[]);
        function getLayoutInfo() external view returns (uint256 publicMemoryOffset, uint256 selectedBuiltins);
    }
}

#[storage]
#[entrypoint]
pub struct GpsStatementVerifier {
    initialized: StorageBool,
    memory_page_fact_registry: StorageAddress,
    verifiers: StorageVec<StorageAddress>,
    verified_facts: StorageMap<FixedBytes<32>, StorageBool>,
    any_fact_registered: StorageBool,
}

impl BootloaderCompiledProgram for GpsStatementVerifier {}

/// Declare that `GpsStatementVerifier` is a contract with the following external methods.
#[public]
impl GpsStatementVerifier {
    pub fn init(
        &mut self,
        memory_page_fact_registry: Address,
        verifiers: Vec<Address>,
    ) -> Result<(), Vec<u8>> {
        require!(!self.initialized.get(), "already initialized");

        self.memory_page_fact_registry
            .set(memory_page_fact_registry);

        for addr in verifiers {
            self.verifiers.push(addr);
        }

        self.initialized.set(true);
        Ok(())
    }

    pub fn verify_proof_and_register(
        &mut self,
        proof_params: Vec<U256>,
        proof: Vec<U256>,
        task_metadata: Vec<U256>,
        cairo_aux_input: Vec<U256>,
        verifier_id: U256,
    ) -> Result<(), Vec<u8>> {
        // fail if it has not been initialized

        require!(self.initialized.get(), "not initialized");

        let verifier_id_usize: usize = match verifier_id.try_into() {
            Ok(val) => val,
            Err(_) => return Err("Verifier Id does not fit in usize".as_bytes().to_vec()),
        };

        require!(
            verifier_id_usize < self.verifiers.len(),
            "cairoVerifierId is out of range."
        );
        let cairo_public_input: &[U256] = &cairo_aux_input[..cairo_aux_input.len() - 2];

        let verifier_address = match self.verifiers.get(verifier_id_usize) {
            Some(verifier) => verifier,
            None => return Err("Verifier not found".as_bytes().to_vec()),
        };

        let verifier_contract = ICairoVerifierContract::new(verifier_address);

        let mut selected_builtins = uint!(151_U256);

        let public_memory_offset_usize: usize = 21;

        require!(
            cairo_public_input.len() > public_memory_offset_usize,
            "Invalid cairoAuxInput length."
        );

        let public_memory_pages: &[U256] = &cairo_public_input[public_memory_offset_usize..];
        let n_pages: usize = public_memory_pages[0]
            .try_into()
            .map_err(|_| "Invalid nPages.")?;
        require!(n_pages < 10000, "Invalid nPages.");

        require!(
            public_memory_pages.len() == n_pages * (PAGE_INFO_SIZE + 1),
            "Invalid publicMemoryPages length."
        );

        let (public_memory_length, memory_hash, product) =
            match Self::register_public_memory_main_page(
                &mut *self,
                &task_metadata,
                &cairo_aux_input,
                &mut selected_builtins,
            ) {
                Ok(val) => val,
                Err(e) => return Err(e),
            };
        // console!("public_memory_length: {}", public_memory_length);
        // console!("memory_hash: {}", memory_hash);
        // console!("product: {}", product);
        require!(
            public_memory_pages[PAGE_INFO_SIZE_OFFSET] == public_memory_length,
            "Invalid size for memory page 0."
        );
        // require!(
        //     public_memory_pages[PAGE_INFO_HASH_OFFSET] == memory_hash,
        //     "Invalid hash for memory page 0."
        // );
        // require!(
        //     public_memory_pages[n_pages * PAGE_INFO_SIZE] == product,
        //     "Invalid cumulative product for memory page 0."
        // );

        let _result = verifier_contract.verify_proof_external(
            &mut *self,
            proof_params,
            proof,
            cairo_public_input.to_vec(),
        )?;

        self.register_gps_facts(
            &task_metadata,
            &public_memory_pages,
            cairo_aux_input[public_input_offsets::OFFSET_OUTPUT_BEGIN_ADDR],
        )?;

        Ok(())
    }

    // pub fn has_registered_fact(&self) -> bool {
    //     self.any_fact_registered.get()
    // }

    // fn is_valid(&self, fact: FixedBytes<32>) -> bool {
    //     self.fact_check(fact)
    // }
}
fn construct_node(
    node_stack: &mut [U256],
    node_stack_len: usize,
    n_nodes: usize,
) -> Result<usize, Vec<u8>> {
    require!(
        n_nodes <= node_stack_len,
        "Invalid value of n_nodes in tree structure."
    );

    // End-offset of the right-most child = end of the parent.
    let new_node_end =
        node_stack[(node_stack_len - 1) * NODE_STACK_ITEM_SIZE + NODE_STACK_OFFSET_END];

    let new_stack_len = node_stack_len - n_nodes;

    // ---  build byte-buffer to hash  ----------------------------------------------------------
    let mut buf = Vec::with_capacity(n_nodes * NODE_STACK_ITEM_SIZE * 32);
    for i in 0..n_nodes {
        let base = (new_stack_len + i) * NODE_STACK_ITEM_SIZE;
        buf.extend_from_slice(&node_stack[base + NODE_STACK_OFFSET_HASH].to_be_bytes::<32>());
        buf.extend_from_slice(&node_stack[base + NODE_STACK_OFFSET_END].to_be_bytes::<32>());
    }

    let new_node_hash = U256::from_be_bytes::<32>(keccak(&buf).into()) + U256::ONE;
    // -----------------------------------------------------------------------------------------

    // Over-write the first child slot with the parent.
    let parent_base = new_stack_len * NODE_STACK_ITEM_SIZE;
    node_stack[parent_base + NODE_STACK_OFFSET_HASH] = new_node_hash;
    node_stack[parent_base + NODE_STACK_OFFSET_END] = new_node_end;

    Ok(new_stack_len + 1) // new length after pushing parent
}

impl GpsStatementVerifier {
    fn register_gps_facts(
        &mut self,
        task_metadata: &[U256],
        public_memory_pages: &[U256],
        output_start_address: U256,
    ) -> Result<(), Vec<u8>> {
        let total_num_pages: usize = public_memory_pages[0]
            .try_into()
            .map_err(|_| "Invalid total number of pages.".as_bytes().to_vec())?;
        let n_tasks: usize = task_metadata[0]
            .try_into()
            .map_err(|_| "Invalid number of tasks.".as_bytes().to_vec())?;

        // node_stack capacity bounded by total_num_pages * NODE_STACK_ITEM_SIZE.
        let mut node_stack: Vec<U256> = vec![U256::ZERO; total_num_pages * NODE_STACK_ITEM_SIZE];
        let mut cur_addr = output_start_address + U256::from(6);
        let mut cur_page = FIRST_CONTINUOUS_PAGE_INDEX;
        let mut task_metadata_offset = METADATA_TASKS_OFFSET;
        let mut page_info_index: usize = PAGE_INFO_SIZE;
        for task in 0..n_tasks {
            let mut cur_offset = U256::ZERO;
            let first_page_of_task = cur_page;

            let n_tree_pairs: usize = task_metadata
                [task_metadata_offset + METADATA_OFFSET_TASK_N_TREE_PAIRS]
                .try_into()
                .map_err(|_| "Invalid number of tree pairs.".as_bytes().to_vec())?;
            let mut node_stack_len: usize = 0;

            for tree_pair in 0..n_tree_pairs {
                let n_pages: usize = task_metadata[task_metadata_offset
                    + METADATA_TASK_HEADER_SIZE
                    + 2 * tree_pair
                    + METADATA_OFFSET_TREE_PAIR_N_PAGES]
                    .try_into()
                    .map_err(|_| "Invalid number of pages.".as_bytes().to_vec())?;

                require!(
                    n_pages < 1usize << 20,
                    "Invalid value of n_pages in tree structure."
                );
                for page in 0..n_pages {
                    // Push page to node stack
                    {
                        let page_addr = public_memory_pages[page_info_index];

                        let page_size =
                            public_memory_pages[page_info_index + PAGE_INFO_SIZE_OFFSET];
                        let page_hash =
                            public_memory_pages[page_info_index + PAGE_INFO_HASH_OFFSET];

                        require!(page_size < MAX_PAGE_SIZE, "Invalid page size.");
                        require!(page_addr == cur_addr, "Invalid page address.");

                        let base = NODE_STACK_ITEM_SIZE * node_stack_len;
                        node_stack[base + NODE_STACK_OFFSET_END] = page_size + cur_offset;
                        node_stack[base + NODE_STACK_OFFSET_HASH] = page_hash;

                        // TODO: handle pageHashesLogData
                        cur_page += 1;
                        node_stack_len += 1;
                        cur_addr += page_size;
                        cur_offset += page_size;
                        page_info_index += PAGE_INFO_SIZE;
                    }
                }

                let n_nodes: usize = task_metadata[task_metadata_offset
                    + METADATA_TASK_HEADER_SIZE
                    + 2 * tree_pair
                    + METADATA_OFFSET_TREE_PAIR_N_NODES]
                    .try_into()
                    .map_err(|_| "Invalid number of nodes.".as_bytes().to_vec())?;
                if n_nodes != 0 {
                    node_stack_len = construct_node(&mut node_stack, node_stack_len, n_nodes)?;
                }
            }

            require!(
                node_stack_len == 1,
                "Node stack must contain exactly one item."
            );
            let program_hash =
                task_metadata[task_metadata_offset + METADATA_OFFSET_TASK_PROGRAM_HASH];

            require!(
                node_stack[NODE_STACK_OFFSET_END] + U256::from(2)
                    == task_metadata[task_metadata_offset + METADATA_OFFSET_TASK_OUTPUT_SIZE],
                "The sum of the page sizes does not match output size."
            );

            let program_output_fact = node_stack[NODE_STACK_OFFSET_HASH];
            // let encoded_data = ;
            let fact = keccak(
                &[
                    program_hash.to_be_bytes::<32>(),
                    program_output_fact.to_be_bytes::<32>(),
                ]
                .concat(),
            );

            // Update taskMetadataOffset.
            task_metadata_offset += METADATA_TASK_HEADER_SIZE + 2 * n_tree_pairs;

            self.register_fact(fact.as_slice());

            // Move curAddr to the output of the next task (skipping the size and hash fields).
            cur_addr += U256::from(2);
        }
        Ok(())
    }

    fn fact_check(&self, fact: FixedBytes<32>) -> bool {
        self.verified_facts.get(fact)
    }

    fn register_fact(&mut self, fact: &[u8]) {
        self.verified_facts
            .insert(FixedBytes::from_slice(fact), true);
        if !self.any_fact_registered.get() {
            self.any_fact_registered.set(true);
        }
    }

    fn register_public_memory_main_page(
        &mut self,
        task_metadata: &[U256],
        aux_input: &[U256],
        selected_builtins: &mut U256,
    ) -> Result<(U256, U256, U256), Vec<u8>> {
        let n_tasks: usize = task_metadata[0].try_into().unwrap();
        require!(n_tasks < 2usize.pow(30), "Invalid number of tasks.");

        // let bootloader_program_size = Self::BOOTLOADER_PROGRAM.len();
        let public_memory_length = Self::BOOTLOADER_PROGRAM.len()
            + 2
            + N_MAIN_ARGS
            + N_MAIN_RETURN_VALUES
            + 3
            + 1
            + 2 * n_tasks;

        let mut public_memory: Vec<U256> = vec![U256::ZERO; 2 * public_memory_length];
        let mut offset = 0;

        for i in 0..Self::BOOTLOADER_PROGRAM.len() {
            public_memory[offset] = U256::from(i + public_input_offsets::INITIAL_PC);
            public_memory[offset + 1] = Self::BOOTLOADER_PROGRAM[i];
            offset += 2;
        }

        {
            let initial_fp = aux_input[public_input_offsets::OFFSET_EXECUTION_BEGIN_ADDR];
            require!(
                initial_fp.gt(&U256::from(2)),
                "Invalid execution begin address."
            );

            public_memory[offset + 0] = initial_fp - U256::from(2);
            public_memory[offset + 1] = initial_fp;
            // Make sure [initial_fp - 1] = 0.
            public_memory[offset + 2] = initial_fp - U256::ONE;
            public_memory[offset + 3] = U256::ZERO;
            offset += 4;

            let return_values_address =
                aux_input[public_input_offsets::OFFSET_EXECUTION_STOP_PTR] - U256::from(N_BUILTINS);
            let mut builtin_segment_info_offset = public_input_offsets::OFFSET_OUTPUT_BEGIN_ADDR;

            for i in 0..N_BUILTINS {
                // Write argument address.
                public_memory[offset] = initial_fp + U256::from(i);
                let return_value_offset = offset + 2 * N_BUILTINS;

                // Write return value address.
                public_memory[return_value_offset] = return_values_address + U256::from(i);

                // Write values.
                if selected_builtins.bit(0) {
                    // Set the argument to the builtin start pointer.
                    public_memory[offset + 1] = aux_input[builtin_segment_info_offset];
                    // Set the return value to the builtin stop pointer.
                    public_memory[return_value_offset + 1] =
                        aux_input[builtin_segment_info_offset + 1];
                    builtin_segment_info_offset += 2;
                } else {
                    // Builtin is not present in layout, set the argument value and return value to 0.
                    public_memory[offset + 1] = U256::ZERO;
                    public_memory[return_value_offset + 1] = U256::ZERO;
                }
                offset += 2;
                *selected_builtins = *selected_builtins >> 1;
            }
            require!(
                *selected_builtins == U256::ZERO,
                "SELECTED_BUILTINS_VECTOR_IS_TOO_LONG"
            );

            // Skip the return values which were already written.
            offset += 2 * N_BUILTINS;
        }

        // Program output.
        {
            let mut output_address = aux_input[public_input_offsets::OFFSET_OUTPUT_BEGIN_ADDR];
            // Force that memory[outputAddress] and memory[outputAddress + 1] contain the
            // bootloader config (which is 2 words size).
            public_memory[offset + 0] = output_address;

            public_memory[offset + 1] = SIMPLE_BOOTLOADER_PROGRAM_HASH;
            public_memory[offset + 2] = output_address + U256::ONE;
            public_memory[offset + 3] = APPLICATION_BOOTLOADER_PROGRAM_HASH;
            // Force that memory[outputAddress + 3] = nTasks.
            public_memory[offset + 4] = output_address + U256::from(2);
            public_memory[offset + 5] = HASHED_SUPPORTED_VERIFIERS;
            public_memory[offset + 6] = output_address + U256::from(3);
            public_memory[offset + 7] = U256::from(n_tasks);

            offset += 8;
            output_address += U256::from(4);

            let mut task_metadata_slice = &task_metadata[METADATA_TASKS_OFFSET..];
            for _task in 0..n_tasks {
                let output_size = task_metadata_slice[METADATA_OFFSET_TASK_OUTPUT_SIZE];
                require!(
                    U256::from(2) <= output_size && output_size < U256::from(1u64 << 30),
                    "Invalid task output size."
                );
                let program_hash = task_metadata_slice[METADATA_OFFSET_TASK_PROGRAM_HASH];
                let n_tree_pairs: usize = task_metadata_slice[METADATA_OFFSET_TASK_N_TREE_PAIRS]
                    .try_into()
                    .unwrap();
                require!(
                    1 <= n_tree_pairs && n_tree_pairs < 2usize.pow(20),
                    "Invalid number of pairs in the Merkle tree structure."
                );
                // Force that memory[outputAddress] = outputSize.
                public_memory[offset + 0] = output_address;
                public_memory[offset + 1] = output_size;
                // Force that memory[outputAddress + 1] = programHash.
                public_memory[offset + 2] = output_address + U256::from(1);
                public_memory[offset + 3] = program_hash;
                offset += 4;
                output_address += output_size;
                let start_index = METADATA_TASK_HEADER_SIZE + 2 * n_tree_pairs;
                task_metadata_slice = &task_metadata_slice[start_index..];
            }
            require!(
                task_metadata_slice.len() == 0,
                "Invalid length of taskMetadata."
            );

            require!(
                aux_input[public_input_offsets::OFFSET_OUTPUT_STOP_PTR] == output_address,
                "Inconsistent program output length."
            );
        }

        require!(
            public_memory.len() == offset,
            "Not all Cairo public inputs were written."
        );

        let z = aux_input[aux_input.len() - 2];
        let alpha = aux_input[aux_input.len() - 1];

        let (_fact, memory_hash_bytes, product) =
            IMemoryPageFactRegistry::new(self.memory_page_fact_registry.get())
                .register_regular_memory_page(&mut *self, public_memory, z, alpha, K_MODULUS)?;

        let memory_hash = U256::from_be_bytes::<32>(memory_hash_bytes.into());
        // console!("memory_hash: {}", memory_hash);
        // console!("product: {}", product);

        Ok((U256::from(public_memory_length), memory_hash, product))
    }
}

const N_BUILTINS: usize = 11;
const N_MAIN_ARGS: usize = N_BUILTINS;
const N_MAIN_RETURN_VALUES: usize = N_BUILTINS;

const METADATA_TASKS_OFFSET: usize = 1;
const METADATA_OFFSET_TASK_OUTPUT_SIZE: usize = 0;
const METADATA_OFFSET_TASK_PROGRAM_HASH: usize = 1;
const METADATA_OFFSET_TASK_N_TREE_PAIRS: usize = 2;
const METADATA_TASK_HEADER_SIZE: usize = 3;

const METADATA_OFFSET_TREE_PAIR_N_PAGES: usize = 0;
const METADATA_OFFSET_TREE_PAIR_N_NODES: usize = 1;

/// Node‑stack bookkeeping.
const NODE_STACK_OFFSET_HASH: usize = 0;
const NODE_STACK_OFFSET_END: usize = 1;
// The size of each node in the node stack.
const NODE_STACK_ITEM_SIZE: usize = 2;

const FIRST_CONTINUOUS_PAGE_INDEX: usize = 1;

const HASHED_SUPPORTED_VERIFIERS: U256 =
    uint!(988080400528720010398639244351885480706475299330001427790099377094461351470_U256);
const K_MODULUS: U256 =
    uint!(0x800000000000011000000000000000000000000000000000000000000000001_U256);

#[cfg(test)]
mod test {
    use core::assert_ne;

    use super::*;
    use stylus_sdk::testing::*;
    #[motsu::test]
    fn test_register_gps_facts() {
        let vm = TestVM::default();
        let mut gpsVerifier: GpsStatementVerifier = GpsStatementVerifier::from(&vm);
        if let Err(e) = gpsVerifier.register_gps_facts(
            &TASK_META_DATA,
            &PUBLIC_MEMORY_PAGES,
            OUTPUT_START_ADDRESS,
        ) {
            let str_err = String::from_utf8(e).unwrap();
            panic!("Error: {:?}", str_err);
        }
    }

    #[motsu::test]
    fn test_register_public_memory_main_page() {
        let vm = TestVM::default();
        let mut gpsVerifier: GpsStatementVerifier = GpsStatementVerifier::from(&vm);
        if let Err(e) = gpsVerifier.register_public_memory_main_page(
            &TASK_META_DATA,
            &AUX_INPUT,
            &mut uint!(151_U256),
        ) {
            let str_err = String::from_utf8(e).unwrap();
            panic!("Error: {:?}", str_err);
        }
    }

    const OUTPUT_START_ADDRESS: U256 = uint!(2174928_U256);

    const TASK_META_DATA: [U256; 96] = uint!([
        17_U256,
        50_U256,
        273279642033703284306509103355536170486431195329675679055627933497997642494_U256,
        1_U256,
        1_U256,
        0_U256,
        305_U256,
        2530337539466159944237001094809327283009177793361359619481044346150483328860_U256,
        1_U256,
        1_U256,
        0_U256,
        708_U256,
        770346231394331402493200980986217737662224545740427952627288191358999988146_U256,
        1_U256,
        1_U256,
        0_U256,
        28_U256,
        273279642033703284306509103355536170486431195329675679055627933497997642494_U256,
        1_U256,
        1_U256,
        0_U256,
        302_U256,
        2530337539466159944237001094809327283009177793361359619481044346150483328860_U256,
        1_U256,
        1_U256,
        0_U256,
        1298_U256,
        2530337539466159944237001094809327283009177793361359619481044346150483328860_U256,
        2_U256,
        2_U256,
        1_U256,
        0_U256,
        2_U256,
        708_U256,
        770346231394331402493200980986217737662224545740427952627288191358999988146_U256,
        1_U256,
        1_U256,
        0_U256,
        19_U256,
        16830627573509542901909952446321116535677491650708854009406762893086223513_U256,
        2_U256,
        2_U256,
        1_U256,
        0_U256,
        2_U256,
        708_U256,
        770346231394331402493200980986217737662224545740427952627288191358999988146_U256,
        1_U256,
        1_U256,
        0_U256,
        39_U256,
        3174901404014912024702042974619036870715605532092680335571201877913899936957_U256,
        2_U256,
        2_U256,
        1_U256,
        0_U256,
        2_U256,
        48_U256,
        273279642033703284306509103355536170486431195329675679055627933497997642494_U256,
        1_U256,
        1_U256,
        0_U256,
        92_U256,
        3485280386001712778192330279103973322645241679001461923469191557000342180556_U256,
        1_U256,
        1_U256,
        0_U256,
        305_U256,
        2530337539466159944237001094809327283009177793361359619481044346150483328860_U256,
        1_U256,
        1_U256,
        0_U256,
        28_U256,
        16830627573509542901909952446321116535677491650708854009406762893086223513_U256,
        2_U256,
        2_U256,
        1_U256,
        0_U256,
        2_U256,
        1275_U256,
        2530337539466159944237001094809327283009177793361359619481044346150483328860_U256,
        2_U256,
        2_U256,
        1_U256,
        0_U256,
        2_U256,
        708_U256,
        770346231394331402493200980986217737662224545740427952627288191358999988146_U256,
        1_U256,
        1_U256,
        0_U256,
        302_U256,
        2530337539466159944237001094809327283009177793361359619481044346150483328860_U256,
        1_U256,
        1_U256,
        0_U256,
    ]);

    const AUX_INPUT: [U256; 115] = uint!([
        0_U256,
        22_U256,
        0_U256,
        65535_U256,
        42800643258479064999893963318903811951182475189843316_U256,
        1_U256,
        5_U256,
        797_U256,
        2174928_U256,
        2174928_U256,
        2181855_U256,
        2181855_U256,
        2219475_U256,
        2280159_U256,
        2378912_U256,
        2804447_U256,
        3645207_U256,
        5425887_U256,
        5480841_U256,
        1_U256,
        290341444919459839_U256,
        23_U256,
        856_U256,
        14468380318782799727436783445422070178171431798249825631545579562828988908621_U256,
        2174934_U256,
        48_U256,
        42801189560190645123465114944559077456907628749593288981822153905262114100909_U256,
        2174984_U256,
        303_U256,
        35927800478883087174372128910245777357775168880057357784089272750787290855368_U256,
        2175289_U256,
        706_U256,
        53818773249146605805203760148464243817493328317582288681042242972375862379089_U256,
        2175997_U256,
        26_U256,
        4781001473911073599741210483908971414696354600481190585518287578748180889241_U256,
        2176025_U256,
        300_U256,
        17247760540749530909579192746800861829911179335370940973148319480623319784318_U256,
        2176327_U256,
        626_U256,
        103818159638018233860808557735902979520870007593457025498920045705066693787205_U256,
        2176953_U256,
        670_U256,
        72775049696212080343088408295912243928026940164410616891412018663718085539476_U256,
        2177625_U256,
        706_U256,
        90266681724064764244848763643866670307652785713336634931119560310425420720755_U256,
        2178333_U256,
        15_U256,
        51304167360997668681668666514589911893867995431849048789223140241516327801323_U256,
        2178348_U256,
        2_U256,
        78338746147236970124700731725183845421594913511827187288591969170390706184117_U256,
        2178352_U256,
        706_U256,
        99099583597112527963626110465134535946741384473177202905074530078888825469782_U256,
        2179060_U256,
        35_U256,
        104144189849385994365231252792732102374708624967992148995747213172695157680627_U256,
        2179095_U256,
        2_U256,
        78338746147236970124700731725183845421594913511827187288591969170390706184117_U256,
        2179099_U256,
        46_U256,
        10361454055143439060831703373321686258683481257699378779299627698786615763757_U256,
        2179147_U256,
        90_U256,
        55694352191983430684712183929020590237443168250085128005872613462795358209893_U256,
        2179239_U256,
        303_U256,
        51982976901920877826213009454364312426850133965572704943125649831229175849478_U256,
        2179544_U256,
        24_U256,
        68039504973223461529279957605519015105590621716141608728150472864832454914730_U256,
        2179568_U256,
        2_U256,
        78338746147236970124700731725183845421594913511827187288591969170390706184117_U256,
        2179572_U256,
        626_U256,
        19409425608100418624498898918515831325182572188497214632404462621533979036176_U256,
        2180198_U256,
        647_U256,
        29536698771322984186807112024525702768651791917899231444155251045091298386285_U256,
        2180847_U256,
        706_U256,
        15151056898033222047375151224464935983168218054484889608869685064949298097993_U256,
        2181555_U256,
        300_U256,
        49235438371197654418606950789676216304768186319349954673166984361413165454964_U256,
        1363528803990236107189942612195482466969376321840443897161606350590887271920_U256,
        2308450783298545958551846280193791909452362757007648321139174291225347699643_U256,
        1695091494064480232702165914146277354513099551130086057337678936234684081512_U256,
        365017238302165678722321489215935032626107283493188092610018849103408589589_U256,
        3344852531197368075962396526099647215478647941733432729668628677382924394123_U256,
        2346107691697741358294718511273616500399433926187774906822242652104288511574_U256,
        909110441427436110460585963752223210244258635812945819892391504520918850616_U256,
        404950900882512254532013742476811570153460251595725211309878552362674765875_U256,
        2071515938899236114604887722716755644431973282886649646338027168788244325723_U256,
        976739199855596611503093274489685000294181241509007998654311647555018018714_U256,
        2881486069264800777911962761266851830905935639417659405703094031285702582334_U256,
        1378659838946977593509185200877912211196950239307015541055713387716915334902_U256,
        2633405774800635621761276850945424343975038566305865618759260718456716056930_U256,
        2688660225284173972481313541842382239056570184358163660938322549333952030874_U256,
        1646889559706269662330209234705652899522395484120980841452823611597167694163_U256,
        899761213235315630293837413044255067686262367970240983814953322088871286874_U256,
        459364284863184456706114364639315033696148237348064556694423309987246650889_U256,
        695348909140372781959262786906478781448773362337762224395617347942944455302_U256,
        2944398636680588638745964273159171196611928286838381941158087300894086485788_U256,
        3466849235131796386731342820094773541503586747290581029170201296391716863418_U256,
        175995597008010738549538521138080395578736634040788647146306452088796215562_U256,
        1284413430990585398121570609170125578355152484704757246459467840579104615694_U256,
        3138407396337160205764296473504219124973562925440288500849754760116183056493_U256,
        1889307229587391548520309518094765161966447786756161259520600117314548314282_U256,
        128717201870591596518410513636207059091228026160841752656809259718526660533_U256
    ]);

    const PUBLIC_MEMORY_PAGES: [U256; 92] = uint!([
        23_U256,
        856_U256,
        14468380318782799727436783445422070178171431798249825631545579562828988908621_U256,
        2174934_U256,
        48_U256,
        42801189560190645123465114944559077456907628749593288981822153905262114100909_U256,
        2174984_U256,
        303_U256,
        35927800478883087174372128910245777357775168880057357784089272750787290855368_U256,
        2175289_U256,
        706_U256,
        53818773249146605805203760148464243817493328317582288681042242972375862379089_U256,
        2175997_U256,
        26_U256,
        4781001473911073599741210483908971414696354600481190585518287578748180889241_U256,
        2176025_U256,
        300_U256,
        17247760540749530909579192746800861829911179335370940973148319480623319784318_U256,
        2176327_U256,
        626_U256,
        103818159638018233860808557735902979520870007593457025498920045705066693787205_U256,
        2176953_U256,
        670_U256,
        72775049696212080343088408295912243928026940164410616891412018663718085539476_U256,
        2177625_U256,
        706_U256,
        90266681724064764244848763643866670307652785713336634931119560310425420720755_U256,
        2178333_U256,
        15_U256,
        51304167360997668681668666514589911893867995431849048789223140241516327801323_U256,
        2178348_U256,
        2_U256,
        78338746147236970124700731725183845421594913511827187288591969170390706184117_U256,
        2178352_U256,
        706_U256,
        99099583597112527963626110465134535946741384473177202905074530078888825469782_U256,
        2179060_U256,
        35_U256,
        104144189849385994365231252792732102374708624967992148995747213172695157680627_U256,
        2179095_U256,
        2_U256,
        78338746147236970124700731725183845421594913511827187288591969170390706184117_U256,
        2179099_U256,
        46_U256,
        10361454055143439060831703373321686258683481257699378779299627698786615763757_U256,
        2179147_U256,
        90_U256,
        55694352191983430684712183929020590237443168250085128005872613462795358209893_U256,
        2179239_U256,
        303_U256,
        51982976901920877826213009454364312426850133965572704943125649831229175849478_U256,
        2179544_U256,
        24_U256,
        68039504973223461529279957605519015105590621716141608728150472864832454914730_U256,
        2179568_U256,
        2_U256,
        78338746147236970124700731725183845421594913511827187288591969170390706184117_U256,
        2179572_U256,
        626_U256,
        19409425608100418624498898918515831325182572188497214632404462621533979036176_U256,
        2180198_U256,
        647_U256,
        29536698771322984186807112024525702768651791917899231444155251045091298386285_U256,
        2180847_U256,
        706_U256,
        15151056898033222047375151224464935983168218054484889608869685064949298097993_U256,
        2181555_U256,
        300_U256,
        49235438371197654418606950789676216304768186319349954673166984361413165454964_U256,
        1363528803990236107189942612195482466969376321840443897161606350590887271920_U256,
        2308450783298545958551846280193791909452362757007648321139174291225347699643_U256,
        1695091494064480232702165914146277354513099551130086057337678936234684081512_U256,
        365017238302165678722321489215935032626107283493188092610018849103408589589_U256,
        3344852531197368075962396526099647215478647941733432729668628677382924394123_U256,
        2346107691697741358294718511273616500399433926187774906822242652104288511574_U256,
        909110441427436110460585963752223210244258635812945819892391504520918850616_U256,
        404950900882512254532013742476811570153460251595725211309878552362674765875_U256,
        2071515938899236114604887722716755644431973282886649646338027168788244325723_U256,
        976739199855596611503093274489685000294181241509007998654311647555018018714_U256,
        2881486069264800777911962761266851830905935639417659405703094031285702582334_U256,
        1378659838946977593509185200877912211196950239307015541055713387716915334902_U256,
        2633405774800635621761276850945424343975038566305865618759260718456716056930_U256,
        2688660225284173972481313541842382239056570184358163660938322549333952030874_U256,
        1646889559706269662330209234705652899522395484120980841452823611597167694163_U256,
        899761213235315630293837413044255067686262367970240983814953322088871286874_U256,
        459364284863184456706114364639315033696148237348064556694423309987246650889_U256,
        695348909140372781959262786906478781448773362337762224395617347942944455302_U256,
        2944398636680588638745964273159171196611928286838381941158087300894086485788_U256,
        3466849235131796386731342820094773541503586747290581029170201296391716863418_U256,
        175995597008010738549538521138080395578736634040788647146306452088796215562_U256,
        1284413430990585398121570609170125578355152484704757246459467840579104615694_U256,
        3138407396337160205764296473504219124973562925440288500849754760116183056493_U256
    ]);
}
