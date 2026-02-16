use rand::Rng;

fn get_available_memory_bandwith_gbps() -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(20.0..120.0)
}

fn calculate_dynamic_draft_length(available_bandwidth: f32) -> u32, {
    let min_bandwidth = 30.0;
    let alpha = 0.15;

    if available_bandwidth < min_bandwidth {
        return 1;
    }
    let raw_length = (available_bandwidth - min_bandwidth);

    let draft_length = raw_length.floor() as u32;
    draft_length.clamp(1, 8)
}

fn main() {
    println!("Starting the HaloSpec Bandwidth-Aware Speculative Scheduler...");
    
    println!("Initializing system telemetry module...");
    let current_bw = get_available_memory_bandwidth_gbps();
    let draft_tokens = calculate_dynamic_draft_length(current_bw);

    println!("Test Run -> OS Memory Bandwidth: {:.2} GB/s | Dynamic Draft Size: {}", current_bw, draft_tokens);
    
    println!("Initializing Lemonade Engine...");
    // TODO: Connect to local AMD Lemonade API on localhost:8000
    
    println!("System is ready. Waiting for dynamic inference requests...");
}