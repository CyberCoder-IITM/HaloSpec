import os
import pandas as pd
import matplotlib.pyplot as plt

CSV_PATH = "results_phase0.csv"
OUT_DIR = os.path.join("docs", "graphs")
os.makedirs(OUT_DIR, exist_ok=True)

def savefig(name: str):
    path = os.path.join(OUT_DIR, name)
    plt.tight_layout()
    plt.savefig(path, dpi=180)
    plt.close()
    print(f"[OK] wrote {path}")

def main():
    if not os.path.exists(CSV_PATH):
        raise FileNotFoundError(f"Missing {CSV_PATH} in repo root.")

    df = pd.read_csv(CSV_PATH)

    # Basic cleanup / types
    df["step"] = df["step"].astype(int)
    df["global_step"] = df["global_step"].astype(int)
    df["latency_ms"] = pd.to_numeric(df["latency_ms"], errors="coerce")
    df["tokens"] = pd.to_numeric(df["tokens"], errors="coerce")
    df["success"] = pd.to_numeric(df["success"], errors="coerce").fillna(0).astype(int)
    if "phase" not in df.columns:
        df["phase"] = "steady"

    # Only successful rows for latency/tokens analysis
    ok = df[df["success"] == 1].copy()

    # ----------------------------
    # 1) Adaptive latency vs step (phase-colored)
    # ----------------------------
    ad = ok[ok["mode"] == "adaptive"].sort_values("step")
    if not ad.empty:
        plt.figure()
        plt.grid(True, linestyle="--", linewidth=0.5, alpha=0.6)
        for phase_name, grp in ad.groupby("phase", sort=False):
            plt.plot(grp["step"], grp["latency_ms"], marker="o", linewidth=1, label=phase_name)
        plt.xlabel("step (adaptive)")
        plt.ylabel("latency (ms)")
        plt.title("Adaptive latency vs step (colored by phase)")
        plt.legend()
        savefig("adaptive_latency_by_step.png")

        # ----------------------------
        # 2) Adaptive draft_length vs step (phase-colored)
        # ----------------------------
        plt.figure()
        plt.grid(True, linestyle="--", linewidth=0.5, alpha=0.6)
        for phase_name, grp in ad.groupby("phase", sort=False):
            plt.plot(grp["step"], grp["draft_length"], marker="o", linewidth=1, label=phase_name)
        plt.xlabel("step (adaptive)")
        plt.ylabel("draft_length")
        plt.title("Adaptive draft_length vs step (colored by phase)")
        plt.legend()
        savefig("adaptive_draft_length_by_step.png")

        # ----------------------------
        # 3) Adaptive latency distribution by phase (boxplot)
        # ----------------------------
        phases = ["steady", "load", "recovery"]
        data = [ad[ad["phase"] == p]["latency_ms"].dropna().values for p in phases if p in ad["phase"].unique()]
        labels = [p for p in phases if p in ad["phase"].unique()]
        if len(data) >= 1:
            plt.figure()
            plt.grid(True, linestyle="--", linewidth=0.5, alpha=0.6)
            plt.boxplot(data, tick_labels=labels, showfliers=False)
            plt.xlabel("phase")
            plt.ylabel("latency (ms)")
            plt.title("Adaptive latency distribution by phase")
            savefig("adaptive_latency_by_phase_boxplot.png")
    else:
        print("[WARN] No adaptive rows found in CSV; skipping adaptive plots.")

    # ----------------------------
    # 4) Fixed modes comparison: avg latency + p95 latency
    # ----------------------------
    fixed_ok = ok[ok["mode"].str.startswith("fixed_")].copy()
    if not fixed_ok.empty:
        # avg latency per mode
        g = fixed_ok.groupby("mode")["latency_ms"]
        avg_lat = g.mean().sort_index()
        p95_lat = g.quantile(0.95).sort_index()

        plt.figure()
        plt.grid(True, linestyle="--", linewidth=0.5, alpha=0.6)
        plt.bar(avg_lat.index, avg_lat.values)
        plt.xticks(rotation=45, ha="right")
        plt.xlabel("mode")
        plt.ylabel("avg latency (ms)")
        plt.title("Fixed modes: average latency")
        savefig("fixed_avg_latency.png")

        plt.figure()
        plt.grid(True, linestyle="--", linewidth=0.5, alpha=0.6)
        plt.bar(p95_lat.index, p95_lat.values)
        plt.xticks(rotation=45, ha="right")
        plt.xlabel("mode")
        plt.ylabel("p95 latency (ms)")
        plt.title("Fixed modes: p95 latency")
        savefig("fixed_p95_latency.png")
    else:
        print("[WARN] No fixed_* rows found; skipping fixed plots.")

    # ----------------------------
    # 5) SLO-aware score comparison (computed from CSV, not from console)
    # score = avg + 0.5*p95 + 0.2*stddev
    # ----------------------------
    def score_for_mode(mode_df: pd.DataFrame):
        lat = mode_df["latency_ms"].dropna()
        if lat.empty:
            return None
        avg = float(lat.mean())
        p95 = float(lat.quantile(0.95))
        sd = float(lat.std(ddof=0))  # match your code's population variance
        return avg + 0.5 * p95 + 0.2 * sd

    scores = {}
    for mode_name, grp in ok.groupby("mode"):
        sc = score_for_mode(grp)
        if sc is not None:
            scores[mode_name] = sc

    if scores:
        s = pd.Series(scores).sort_index()
        plt.figure()
        plt.grid(True, linestyle="--", linewidth=0.5, alpha=0.6)
        plt.bar(s.index, s.values)
        plt.xticks(rotation=45, ha="right")
        plt.xlabel("mode")
        plt.ylabel("score (lower is better)")
        plt.title("SLO-aware score by mode (from CSV)")
        savefig("score_by_mode.png")
    else:
        print("[WARN] No scores computed; skipping score plot.")

    print("\nDone.")

if __name__ == "__main__":
    main()