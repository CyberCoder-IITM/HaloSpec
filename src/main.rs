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
            endpoint: "http://localhost:8000/api/v1/chat/completions".to_string(),
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
            "max_tokens": 192,
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

                        let debug = std::env::var("HALOSPEC_DEBUG_JSON")

                             .ok()                         
                             .as_deref() == Some("1");

                        if debug {
                          println!("\n[DEBUG] Full response JSON:\n{}\n", res_json);
                        }


                        let reply = res_json["choices"][0]["message"]["content"]
                            .as_str()
                            .and_then(|s| if s.trim().is_empty() { None } else { Some(s) })
                            .or_else(|| {
                                res_json["choices"][0]["message"]["reasoning_content"]
                                    .as_str()
                                    .and_then(|s| if s.trim().is_empty() { None } else { Some(s) })
                            })
                            .or_else(|| {
                                res_json["choices"][0]["text"]
                                    .as_str()
                                    .and_then(|s| if s.trim().is_empty() { None } else { Some(s) })
                            })
                            .unwrap_or("[EMPTY RESPONSE]")
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
    let low = 9_000;//fast system
    let high = 22_000; // overloaded / slow

    match last_latency_ms {
        None => current, // first run, keep initial
        Some(lat) if lat < low => (current + 1).clamp(1, 8),
        Some(lat) if lat > high => current.saturating_sub(1).clamp(1, 8),
        Some(_) => {
            if current < 6 {
                (current + 1).clamp(1, 8)
            } else {
                current
            }
        }
    }
}

/// Runs one benchmark mode and logs CSV
fn run_mode(
    engine: &LemonadeEngine,
    mode: &str,
    steps: usize,
    prompt: &str,
    fixed: Option<u32>,
) -> ModeStats {
    println!("\n==============================");
    println!("MODE: {}", mode);
    println!("==============================\n");

    let mut last_latency: Option<u128> = None;
    let mut draft_len: u32 = fixed.unwrap_or(4);


    let mut stats = ModeStats {
        mode: mode.to_string(),
        steps,
        successes: 0,
        failures: 0,
        latencies_ms: Vec::with_capacity(steps),
    };

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("halospec_results.csv")
        .expect("failed to open halospec_results.csv");

    for step in 1..=steps {

        let chosen = match fixed {
            Some(v) => v,
            None => adaptive_draft_length(last_latency, draft_len),
        };

        println!("[Step {}] chosen_draft_length={}", step, chosen);

        let (ok, latency_ms, preview) = engine.generate_with_retry(prompt, chosen);

        if ok {
            println!("[OK] latency={}ms | reply_preview={}", latency_ms, preview);
            stats.successes += 1;
            stats.latencies_ms.push(latency_ms);
            last_latency = Some(latency_ms);
            draft_len = chosen;
        } else {
            println!("[FAIL] request failed after retries");
            stats.failures += 1;
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

    stats
}

#[derive(Debug, Clone)]
struct ModeStats {
    mode: String,
    steps: usize,
    successes: usize,
    failures: usize,
    latencies_ms: Vec<u128>, // only successful latencies
}

impl ModeStats {
    fn success_rate(&self) -> f64 {
        if self.steps == 0 { return 0.0; }
        (self.successes as f64) / (self.steps as f64)
    }

    fn avg(&self) -> Option<f64> {
        if self.latencies_ms.is_empty() { return None; }
        let sum: u128 = self.latencies_ms.iter().sum();
        Some(sum as f64 / self.latencies_ms.len() as f64)
    }

    fn min(&self) -> Option<u128> {
        self.latencies_ms.iter().copied().min()
    }

    fn max(&self) -> Option<u128> {
        self.latencies_ms.iter().copied().max()
    }

    fn median(&self) -> Option<u128> {
        percentile_u128(&self.latencies_ms, 50.0)
    }

    fn p95(&self) -> Option<u128> {
        percentile_u128(&self.latencies_ms, 95.0)
    }
}

fn percentile_u128(values: &Vec<u128>, pct: f64) -> Option<u128> {
    if values.is_empty() { return None; }
    let mut v = values.clone();
    v.sort_unstable();
    // nearest-rank method
    let n = v.len();
    let rank = ((pct / 100.0) * (n as f64)).ceil() as usize;
    let idx = rank.saturating_sub(1).min(n - 1);
    Some(v[idx])
}

fn fmt_opt_ms(v: Option<u128>) -> String {
    match v {
        Some(x) => format!("{} ms", x),
        None => "-".to_string(),
    }
}

fn fmt_opt_avg(v: Option<f64>) -> String {
    match v {
        Some(x) => format!("{:.1} ms", x),
        None => "-".to_string(),
    }
}

fn print_summary(stats: &[ModeStats]) {
    println!("\n==============================");
    println!("HaloSpec Benchmark Summary");
    println!("==============================\n");

    println!(
        "{:<10} {:>6} {:>9} {:>9} {:>12} {:>12} {:>12} {:>12} {:>10}",
        "mode", "steps", "success", "fail", "avg", "median", "p95", "min", "max"
    );

    for s in stats {
        println!(
            "{:<10} {:>6} {:>8.1}% {:>9} {:>12} {:>12} {:>12} {:>12} {:>10}",
            s.mode,
            s.steps,
            s.success_rate() * 100.0,
            s.failures,
            fmt_opt_avg(s.avg()),
            fmt_opt_ms(s.median()),
            fmt_opt_ms(s.p95()),
            fmt_opt_ms(s.min()),
            fmt_opt_ms(s.max()),
        );
    }

    // Identify best avg (only among modes with avg available)
    let mut best: Option<(&str, f64)> = None;
    for s in stats {
        if let Some(a) = s.avg() {
            match best {
                None => best = Some((s.mode.as_str(), a)),
                Some((_, best_a)) if a < best_a => best = Some((s.mode.as_str(), a)),
                _ => {}
            }
        }
    }

    if let Some((m, a)) = best {
        println!("\nWinner (lowest avg latency): {} at {:.1} ms", m, a);
    }
}

fn main() {
    println!("Starting HaloSpec: Adaptive Speculative Scheduler Benchmark...");

    // Use a stable prompt for benchmarking (content doesn't matter much)
    let prompt = "Explain speculative decoding in ONE sentence. Output ONLY the final sentence.";

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
    let s1 = run_mode(&engine, "fixed_1", 10, prompt, Some(1));
    let s2 = run_mode(&engine, "fixed_8", 10, prompt, Some(8));
    let s3 = run_mode(&engine, "adaptive", 15, prompt, None);

    print_summary(&[s1, s2, s3]);

    println!("\nDone. Results saved to halospec_results.csv");
}