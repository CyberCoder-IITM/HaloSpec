use rand::Rng;
use reqwest::blocking::Client;
use serde_json::json;
use std::fs::OpenOptions;
use std::io::Write;
use std::thread::sleep;
use std::time::{Duration, Instant};


#[derive(Clone)]
struct LemonadeEngine {
    target_model: String,
    client: Client,
    endpoint: String,
}

impl LemonadeEngine {
    fn new(target: &str) -> Self {
        println!("[Lemonade API] Connecting to Local Server at localhost:8000...");
        // BUILDER: Setting a 30-second timeout to handle high-bandwidth speculative batches
        let custom_client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| Client::new());

        LemonadeEngine {
            target_model: target.to_string(),
            client: custom_client,
        }
    }

    /// Send a request with retries and return (success, latency_ms, reply_preview)
    fn generate_with_retry(&self, prompt: &str, draft_length: u32) -> (bool, u128, String) {
        let payload = json!({
            "model": self.target_model,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "stream": false,
            "speculative_draft_length": draft_length
        });

        let mut backoff_ms = 400u64;

        for attempt in 1..=3 {
            println!(
                "[HaloSpec] POST attempt {}/3 | draft_length={}",
                attempt, draft_length
            );

            let start = Instant::now();
            let res = self
                .client
                .post(&self.endpoint)
                .json(&payload)
                .send();

            match res {
                Ok(response) => {
                    let latency = start.elapsed().as_millis();

                    if response.status().is_success() {
                        let res_json: serde_json::Value = response.json().unwrap_or_default();
                        let reply = res_json["choices"][0]["message"]["content"]
                            .as_str()
                            .unwrap_or("")
                            .trim()
                            .to_string();

                        // keep output short for logs
                        let preview = reply.chars().take(140).collect::<String>();
                        return (true, latency, preview);
                    } else {
                        println!(
                            "[API Error] status={} | latency={}ms",
                            response.status(),
                            latency
                        );
                    }
                }
                Err(e) => {
                    println!("[Connection Error] {} (attempt {})", e, attempt);
                }
            }

            // retry with exponential backoff
            println!("[HaloSpec] Backoff {}ms then retry...", backoff_ms);
            sleep(Duration::from_millis(backoff_ms));
            backoff_ms *= 2;
        }

        (false, 0, "FAILED".to_string())
    }
}

/// Adaptive controller: uses last latency to choose draft length
fn adaptive_draft_length(last_latency_ms: Option<u128>, current: u32) -> u32 {
    // You can tune these thresholds after seeing real data
    let low = 800;   // fast system
    let high = 2500; // overloaded / slow

    match last_latency_ms {
        None => current, // first run, keep initial
        Some(lat) if lat < low => (current + 1).clamp(1, 8),
        Some(lat) if lat > high => current.saturating_sub(1).clamp(1, 8),
        Some(_) => current, // stable zone
    }
}

/// Runs one benchmark mode and logs CSV
fn run_mode(engine: &LemonadeEngine, mode: &str, steps: usize, prompt: &str, fixed: Option<u32>) {
    println!("\n==============================");
    println!("MODE: {}", mode);
    println!("==============================\n");

    let mut rng = rand::thread_rng();

    let mut last_latency: Option<u128> = None;
    let mut draft_len: u32 = fixed.unwrap_or(4);

    // open CSV append
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("halospec_results.csv")
        .expect("failed to open halospec_results.csv");

    for step in 1..=steps {
        // Simulate some variability in the system load by random sleep (optional)
        let jitter = rng.gen_range(200..700);
        sleep(Duration::from_millis(jitter));

        let chosen = match fixed {
            Some(v) => v,
            None => adaptive_draft_length(last_latency, draft_len),
        };

        println!("[Step {}] chosen_draft_length={}", step, chosen);

        let (ok, latency_ms, preview) = engine.generate_with_retry(prompt, chosen);

        if ok {
            println!("[OK] latency={}ms | reply_preview={}", latency_ms, preview);
            last_latency = Some(latency_ms);
            draft_len = chosen;
        } else {
            println!("[FAIL] request failed after retries");
            // If fail, fall back to conservative
            draft_len = 1;
        }

        // CSV: timestamp-ish (step), mode, draft_length, success, latency_ms
        writeln!(
            file,
            "{},{},{},{},{}",
            step,
            mode,
            chosen,
            if ok { 1 } else { 0 },
            latency_ms
        )
        .ok();

        // cooldown for local server stability
        sleep(Duration::from_secs(2));
    }
}


fn main() {
    println!("Starting HaloSpec: Adaptive Speculative Scheduler Benchmark...");

    // Use a stable prompt for benchmarking (content doesn't matter much)
    let prompt = "Explain speculative decoding in one sentence.";

    let engine = LemonadeEngine::new("Qwen3-0.6B-GGUF");

    // Write CSV header once (if file is new)
    if std::fs::metadata("halospec_results.csv").is_err() {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("halospec_results.csv")
            .unwrap();
        writeln!(file, "step,mode,draft_length,success,latency_ms").ok();
    }

    // Baselines + adaptive
    run_mode(&engine, "fixed_1", 10, prompt, Some(1));
    run_mode(&engine, "fixed_8", 10, prompt, Some(8));
    run_mode(&engine, "adaptive", 15, prompt, None);

    println!("\nDone. Results saved to halospec_results.csv");
}