trait MemoryAccessUtils {
    fn get_channel_ptr() -> usize {
        MM_CHANNEL
    }

    fn get_merkle_queue_ptr(ctx: &[U256]) -> usize {
        MM_MERKLE_QUEUE
    }

    fn get_fri_step_sizes(ctx: &[U256]) -> Vec<U256> {
        // Lyubo: Use MM_FRI_STEP_SIZES_PTR is 294 = place of the array of FRI step sizes
        ctx[294]
    }
}