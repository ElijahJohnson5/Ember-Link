// One day is max timeout
pub fn calculate_backoff(attempt: u64, random_seed: f32) -> u32 {
    ((std::cmp::min(86_400_000, attempt * attempt * 1000) as f32) + 2000.0 * random_seed).floor()
        as u32
}
