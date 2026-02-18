use reqwest::blocking::Client;
use serde_json::json;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::{Duration, Instant};
const CSV_PATH: &str = "results_phase0.csv";
const WARMUP_STEPS: usize = 5; // per mode, not logged, not counted

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
    fn generate_with_retry(&self, prompt: &str, draft_length: u32) -> (bool, u128, u64, String) {
        let payload = json!({
            "model": self.target_model,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "stream": false,
            "max_tokens":64,
            "temperature": 0.0,
            "top_p": 1.0,
            "stop":  ["</think>"],
            "speculative_draft_length": draft_length
        });

        let mut backoff_ms = 400u64;

        for attempt in 1..=3 {
            println!(
                "[HaloSpec] POST attempt {}/3 | draft_length={}",
                attempt, draft_length
            );

            let start = Instant::now();
            let res = self.client.post(&self.endpoint).json(&payload).send();

            match res {
                Ok(response) => {
                    let latency = start.elapsed().as_millis();

                    if response.status().is_success() {
                        let res_json: serde_json::Value = response.json().unwrap_or_default();

                        let debug =
                            std::env::var("HALOSPEC_DEBUG_JSON").ok().as_deref() == Some("1");

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
                        let cleaned = sanitize_reply(reply.clone());
                        let preview = cleaned.chars().take(140).collect::<String>();

                        let tokens = res_json["usage"]["completion_tokens"]
                            .as_u64()
                            .unwrap_or(64); // fallback if usage missing

                        return (true, latency, tokens, preview);
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

        (false, 0, 0, "FAILED".to_string())
    }
}

fn adaptive_draft_length(
    last_latency_ms: Option<u128>,
    current: u32,
    low: u128,
    high: u128,
) -> u32 {
    match last_latency_ms {
        None => current,
        Some(lat) if lat < low => (current + 1).min(8), // keep
        Some(lat) if lat > high => current.saturating_sub(2).clamp(1, 8),
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
    global_step: &mut u64,
) -> ModeStats {
    println!("\n==============================");
    println!("MODE: {}", mode);
    println!("==============================\n");

    let load_on = std::env::var("HALOSPEC_LOAD").ok().as_deref() == Some("1");

    // Warmup phase: prime caches / stabilize Lemonade server (not logged, not counted)
    let mut warm_lat: Vec<u128> = Vec::with_capacity(WARMUP_STEPS);

    for w in 1..=WARMUP_STEPS {
        let warm_draft = fixed.unwrap_or(4); // stable warmup choice
        println!("[Warmup {}] draft_length={}", w, warm_draft);

        let (ok, latency_ms, _preview_tokens, _preview) =
            engine.generate_with_retry(prompt, warm_draft);
        if ok {
            warm_lat.push(latency_ms);
        }

        sleep(Duration::from_secs(1));
    }
    println!("[Warmup done] Starting measured steps...\n");

    let mut load_handle: Option<std::thread::JoinHandle<()>> = None;
    let load_active = Arc::new(AtomicBool::new(false));
    let load_active_clone = load_active.clone();
    // Adaptive calibration: set thresholds based on warmup distribution (per run, per machine)
    let mut low_thr: u128 = 9_000;
    let mut high_thr: u128 = 22_000;

    if mode == "adaptive" && !warm_lat.is_empty() {
        let p50 = percentile_u128(&warm_lat, 50.0).unwrap_or(warm_lat[0]);
        let p95 = percentile_u128(&warm_lat, 95.0).unwrap_or(p50);

        low_thr = ((p50 as f64) * 0.85) as u128;
        high_thr = ((p95 as f64) * 1.05) as u128;

        println!(
            "[Adaptive Calib] warmup_p50={}ms warmup_p95={}ms => low={}ms high={}ms",
            p50, p95, low_thr, high_thr
        );

        // if mode == "adaptive" && load_on {
        //     println!("[Load] Spawning CPU burner for 30s to simulate contention...");
        //     load_handle = Some(spawn_cpu_burner(30));
        // }
        // We'll start load at a specific measured step (see per-step logic below)
    }

    let mut last_latency: Option<u128> = None;
    let mut latency_ema: Option<f64> = None;
    let mut draft_len: u32 = fixed.unwrap_or(4);

    let mut stats = ModeStats {
        mode: mode.to_string(),
        steps,
        successes: 0,
        failures: 0,
        latencies_ms: Vec::with_capacity(steps),
        tokens_generated: Vec::with_capacity(steps),
        draft_lengths: Vec::with_capacity(steps),
    };

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("results_phase0.csv")
        .expect("failed to open results_phase0.csv");

    for step in 1..=steps {
        const LOAD_START_STEP: usize = 6;

        if mode == "adaptive" && load_on && step == LOAD_START_STEP && load_handle.is_none() {
            println!(
                "[Load] Spawning CPU burner for 30s starting at step {}...",
                LOAD_START_STEP
            );

            load_active.store(true, Ordering::Relaxed);
            let load_active_done = load_active.clone();

            load_handle = Some(std::thread::spawn(move || {
                let h = spawn_cpu_burner(30);
                let _ = h.join();
                load_active_done.store(false, Ordering::Relaxed);
            }));
        }

        *global_step += 1;

        let chosen = match fixed {
            Some(v) => v,
            None => adaptive_draft_length(last_latency, draft_len, low_thr, high_thr),
        };

        println!("[Step {}] chosen_draft_length={}", step, chosen);
        stats.draft_lengths.push(chosen);

        let (ok, latency_ms, tokens, preview) = engine.generate_with_retry(prompt, chosen);

        if ok {
            println!(
                "[OK] latency={}ms | tokens={} | reply_preview={}",
                latency_ms, tokens, preview
            );
            stats.successes += 1;
            stats.latencies_ms.push(latency_ms);
            stats.tokens_generated.push(tokens);
            let alpha = 0.4; // tune 0.3–0.5 if needed
            latency_ema = Some(match latency_ema {
                None => latency_ms as f64,
                Some(prev) => alpha * latency_ms as f64 + (1.0 - alpha) * prev,
            });
            last_latency = Some(latency_ema.unwrap() as u128);
            draft_len = chosen;
        } else {
            println!("[FAIL] request failed after retries");
            stats.failures += 1;
            // If fail, fall back to conservative
            draft_len = 1;
        }

        let phase = if mode == "adaptive" && load_on {
            if step < LOAD_START_STEP {
                "steady"
            } else if load_active_clone.load(Ordering::Relaxed) {
                "load"
            } else {
                "recovery"
            }
        } else {
            "steady"
        };

        // CSV: timestamp-ish (step), mode, draft_length, success, latency_ms
        writeln!(
            file,
            "{},{},{},{},{},{},{},{}",
            *global_step,
            step,
            mode,
            phase,
            chosen,
            if ok { 1 } else { 0 },
            latency_ms,
            tokens
        )
        .ok();

        // cooldown for local server stability
        sleep(Duration::from_secs(2));
    }

    if let Some(h) = load_handle {
        let _ = h.join();
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
    tokens_generated: Vec<u64>,
    draft_lengths: Vec<u32>,
}

impl ModeStats {
    fn success_rate(&self) -> f64 {
        if self.steps == 0 {
            return 0.0;
        }
        (self.successes as f64) / (self.steps as f64)
    }

    fn avg(&self) -> Option<f64> {
        if self.latencies_ms.is_empty() {
            return None;
        }
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
    fn throughput(&self) -> Option<f64> {
        if self.latencies_ms.is_empty() {
            return None;
        }

        let total_tokens: u64 = self.tokens_generated.iter().sum();
        let total_ms: u128 = self.latencies_ms.iter().sum();

        if total_ms == 0 {
            return None;
        }

        Some(total_tokens as f64 / (total_ms as f64 / 1000.0))
    }

    fn stddev(&self) -> Option<f64> {
        let avg = self.avg()?; // returns None if no latencies

        if self.latencies_ms.len() < 2 {
            return Some(0.0); // not enough samples to show spread
        }

        let variance = self
            .latencies_ms
            .iter()
            .map(|&x| {
                let d = x as f64 - avg;
                d * d
            })
            .sum::<f64>()
            / (self.latencies_ms.len() as f64);

        Some(variance.sqrt())
    }

    fn score(&self) -> Option<f64> {
        let avg = self.avg()?; // ms
        let p95 = self.p95()? as f64; // ms
        let sd = self.stddev()?; // ms
        Some(avg + 0.5 * p95 + 0.2 * sd)
    }

    fn draft_change_count(&self) -> usize {
        if self.draft_lengths.len() < 2 {
            return 0;
        }
        self.draft_lengths
            .windows(2)
            .filter(|w| w[0] != w[1])
            .count()
    }

    // Returns the first step index (1-based) where draft length stays constant for `k` steps.
    // Example: k=5 means "5 consecutive identical draft lengths".
    fn convergence_step(&self, k: usize) -> Option<usize> {
        if k == 0 || self.draft_lengths.len() < k {
            return None;
        }
        for end in (k - 1)..self.draft_lengths.len() {
            let start = end + 1 - k;
            let slice = &self.draft_lengths[start..=end];
            if slice.iter().all(|&x| x == slice[0]) {
                return Some(end + 1); // 1-based "step"
            }
        }
        None
    }
}

fn percentile_u128(values: &Vec<u128>, pct: f64) -> Option<u128> {
    if values.is_empty() {
        return None;
    }
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

fn fmt_opt_tps(v: Option<f64>) -> String {
    match v {
        Some(x) => format!("{:.1} tok/s", x),
        None => "-".to_string(),
    }
}

fn fmt_opt_stddev(v: Option<f64>) -> String {
    match v {
        Some(x) => format!("{:.1} ms", x),
        None => "-".to_string(),
    }
}

fn fmt_opt_score(v: Option<f64>) -> String {
    match v {
        Some(x) => format!("{:.1}", x),
        None => "-".to_string(),
    }
}

fn sanitize_reply(mut s: String) -> String {
    let t = s.trim();

    // 1) Remove <think> blocks if present
    if let Some(end) = t.find("</think>") {
        s = t[end + "</think>".len()..].trim().to_string();
    } else {
        s = t.to_string();
    }

    let parts: Vec<&str> = s
        .split(|c| c == '.' || c == '!' || c == '?')
        .map(|x| x.trim())
        .filter(|x| !x.is_empty())
        .collect();

    if parts.len() >= 2 {
        // Re-add a period to look like a sentence.
        format!("{}.", parts[parts.len() - 1])
    } else {
        s
    }
}

fn print_summary(stats: &[ModeStats]) {
    println!("\n==============================");
    println!("HaloSpec Benchmark Summary");
    println!("==============================\n");

    println!(
        "{:<10} {:>6} {:>8.1}% {:>9} {:>12} {:>12} {:>12} {:>10} {:>10} {:>12} {:>14} {:>10}",
        "mode",
        "steps",
        "success",
        "fail",
        "avg",
        "median",
        "p95",
        "min",
        "max",
        "stddev",
        "throughput",
        "score"
    );

    for s in stats {
        println!(
            "{:<10} {:>6} {:>8.1}% {:>9} {:>12} {:>12} {:>12} {:>10} {:>10} {:>12} {:>14} {:>10}",
            s.mode,
            s.steps,
            s.success_rate() * 100.0,
            s.failures,
            fmt_opt_avg(s.avg()),
            fmt_opt_ms(s.median()),
            fmt_opt_ms(s.p95()),
            fmt_opt_ms(s.min()),
            fmt_opt_ms(s.max()),
            fmt_opt_stddev(s.stddev()),
            fmt_opt_tps(s.throughput()),
            fmt_opt_score(s.score()),
        );
    }

    // Adaptive controller behavior summary (convergence + oscillation)
    for s in stats {
        if s.mode == "adaptive" {
            let changes = s.draft_change_count();
            let conv = s.convergence_step(5);
            match conv {
                Some(step) => println!(
                    "\n[Adaptive Behavior] draft_length changes={} | converged_at_step={} (k=5)",
                    changes, step
                ),
                None => println!(
                    "\n[Adaptive Behavior] draft_length changes={} | no convergence within run (k=5)",
                    changes
                ),
            }
        }
    }

    // Identify best score (lower is better): avg + 0.5*p95 + 0.2*stddev
    let mut best: Option<(&str, f64)> = None;
    for s in stats {
        if let Some(sc) = s.score() {
            match best {
                None => best = Some((s.mode.as_str(), sc)),
                Some((_, best_sc)) if sc < best_sc => best = Some((s.mode.as_str(), sc)),
                _ => {}
            }
        }
    }

    if let Some((m, sc)) = best {
        println!("\nWinner (lowest SLO-aware score): {} at {:.1}", m, sc);
    }
}

fn spawn_cpu_burner(duration_secs: u64) -> std::thread::JoinHandle<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    std::thread::spawn(move || {
        let start = Instant::now();

        while r.load(Ordering::Relaxed) {
            // burn CPU with some deterministic arithmetic (kept by black_box)
            let mut x: u64 = 0;
            for i in 0..50_000 {
                x = x
                    .wrapping_add(i)
                    .wrapping_mul(1664525)
                    .wrapping_add(1013904223);
            }
            std::hint::black_box(x);

            if start.elapsed().as_secs() >= duration_secs {
                r.store(false, Ordering::Relaxed);
            }
        }
    })
}

fn main() {
    println!("Starting HaloSpec: Adaptive Speculative Scheduler Benchmark...");

    // Use a stable prompt for benchmarking (content doesn't matter much)
    let prompt = "Explain speculative decoding in exactly ONE sentence. Start immediately with the definition (no preface).";
    let engine = LemonadeEngine::new("Qwen3-0.6B-GGUF");
    let mut global_step: u64 = 0;

    // Write CSV header once (if file is new)
    if std::fs::metadata("results_phase0.csv").is_err() {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("results_phase0.csv")
            .unwrap();
        writeln!(
            file,
            "global_step,step,mode,phase,draft_length,success,latency_ms,tokens"
        )
        .ok();
    }

    // Sweep fixed draft lengths 1..=8, then adaptive
    let mut all_stats: Vec<ModeStats> = Vec::new();

    for d in 1..=8u32 {
        let mode = format!("fixed_{}", d);
        let s = run_mode(&engine, &mode, 10, prompt, Some(d), &mut global_step);
        all_stats.push(s);
    }

    let adaptive_stats = run_mode(&engine, "adaptive", 15, prompt, None, &mut global_step);
    all_stats.push(adaptive_stats);

    print_summary(&all_stats);

    println!("\nDone. Results saved to results_phase0.csv");
}
