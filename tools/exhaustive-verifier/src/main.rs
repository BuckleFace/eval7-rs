use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub use eval7_rs::Card;
use eval7_rs::Evaluator;

#[path = "../../../src/canonical.rs"]
mod canonical;
#[path = "../../../src/direct.rs"]
mod direct;
#[path = "../../../src/formula.rs"]
mod formula;
#[allow(dead_code)]
#[path = "../../../src/rank.rs"]
mod rank;
#[path = "../../../src/state.rs"]
mod state;
#[path = "../../../src/tables.rs"]
mod tables;

use canonical::canonical_key_7_by_suit_masks;
use direct::evaluate_7_direct;
use formula::Canonical7FormulaIndexer;

const EXPECTED_HAND_COUNT: u64 = 133_784_560;
const PAYLOAD_SHA256: &str = "ce0ec3e9be1332a76d74cf47368dd37953cef6e097c31cfb4a54062c8b83b774";
const REPRODUCE_COMMAND: &str = "cargo run --release --manifest-path tools/exhaustive-verifier/Cargo.toml -- --report docs/verification/exhaustive-v1.md";

#[derive(Debug)]
struct Mismatch {
    cards: [Card; 7],
    canonical_key: u64,
    formula_index: usize,
    embedded_rank: u16,
    direct_rank: u16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let report_path = parse_report_path()?;
    let evaluator = Evaluator::new();
    let indexer = Canonical7FormulaIndexer::new();
    let deck: [Card; 52] =
        std::array::from_fn(|id| Card::new(id as u8).expect("deck id should be valid"));

    let started = Instant::now();
    let mut evaluated = 0u64;
    let mut mismatch = None;

    'all_hands: for a in 0..46 {
        for b in (a + 1)..47 {
            for c in (b + 1)..48 {
                for d in (c + 1)..49 {
                    for e in (d + 1)..50 {
                        for f in (e + 1)..51 {
                            for g in (f + 1)..52 {
                                let cards = [
                                    deck[a], deck[b], deck[c], deck[d], deck[e], deck[f], deck[g],
                                ];
                                let embedded = evaluator.evaluate_7(cards).map_err(|error| {
                                    format!(
                                        "production evaluator rejected a distinct hand: {error}"
                                    )
                                })?;
                                let direct = evaluate_7_direct(cards);
                                evaluated += 1;

                                if embedded.raw() != direct.raw() {
                                    let key = canonical_key_7_by_suit_masks(cards);
                                    mismatch = Some(Mismatch {
                                        cards,
                                        canonical_key: key,
                                        formula_index: indexer.index_for_key(key),
                                        embedded_rank: embedded.raw(),
                                        direct_rank: direct.raw(),
                                    });
                                    break 'all_hands;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let elapsed = started.elapsed();
    if let Some(details) = &mismatch {
        eprintln!(
            "mismatch after {evaluated} hands: cards={:?}, canonical_key={}, formula_index={}, embedded_rank={}, direct_rank={}",
            details.cards,
            details.canonical_key,
            details.formula_index,
            details.embedded_rank,
            details.direct_rank
        );
    }

    let metadata = Metadata::collect();
    let report = build_report(
        evaluated,
        elapsed.as_secs_f64(),
        mismatch.as_ref(),
        &metadata,
    );
    write_report(&report_path, &report)?;
    println!("report: {}", report_path.display());

    if let Some(details) = mismatch {
        return Err(format!(
            "rank mismatch for {:?}: embedded={}, direct={}",
            details.cards, details.embedded_rank, details.direct_rank
        )
        .into());
    }
    if evaluated != EXPECTED_HAND_COUNT {
        return Err(format!("evaluated {evaluated} hands, expected {EXPECTED_HAND_COUNT}").into());
    }

    println!(
        "verified {evaluated} hands with zero mismatches in {:.3} seconds",
        elapsed.as_secs_f64()
    );
    Ok(())
}

fn parse_report_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    match (args.next(), args.next(), args.next()) {
        (Some(flag), Some(path), None) if flag == "--report" => Ok(PathBuf::from(path)),
        _ => Err(format!("usage: {REPRODUCE_COMMAND}").into()),
    }
}

struct Metadata {
    rustc: String,
    platform: String,
    source_commit: String,
    worktree_dirty: bool,
    unix_timestamp: u64,
}

impl Metadata {
    fn collect() -> Self {
        let rustc = command_output("rustc", &["-vV"]);
        let source_commit = command_output("git", &["rev-parse", "HEAD"]);
        let worktree_dirty = !command_output("git", &["status", "--porcelain"]).is_empty();
        let platform = format!(
            "{} / {} / {}",
            env::consts::OS,
            env::consts::ARCH,
            env::consts::FAMILY
        );
        let unix_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after Unix epoch")
            .as_secs();
        Self {
            rustc,
            platform,
            source_commit,
            worktree_dirty,
            unix_timestamp,
        }
    }
}

fn command_output(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .unwrap_or_else(|| "unavailable".to_owned())
}

fn build_report(
    evaluated: u64,
    elapsed_seconds: f64,
    mismatch: Option<&Mismatch>,
    metadata: &Metadata,
) -> String {
    let successful = mismatch.is_none() && evaluated == EXPECTED_HAND_COUNT;
    let mut report = String::new();
    writeln!(report, "# Eval7 exhaustive verification report\n").unwrap();
    writeln!(
        report,
        "- Result: **{}**",
        if successful { "PASS" } else { "FAIL" }
    )
    .unwrap();
    writeln!(report, "- Seven-card hands evaluated: `{evaluated}`").unwrap();
    writeln!(report, "- Expected hand count: `{EXPECTED_HAND_COUNT}`").unwrap();
    writeln!(
        report,
        "- Mismatches: `{}`",
        usize::from(mismatch.is_some())
    )
    .unwrap();
    writeln!(report, "- Elapsed seconds: `{elapsed_seconds:.6}`").unwrap();
    writeln!(report, "- Platform: `{}`", metadata.platform).unwrap();
    writeln!(report, "- Source commit: `{}`", metadata.source_commit).unwrap();
    writeln!(report, "- Worktree dirty: `{}`", metadata.worktree_dirty).unwrap();
    writeln!(
        report,
        "- Run Unix timestamp: `{}`",
        metadata.unix_timestamp
    )
    .unwrap();
    writeln!(report, "- Embedded payload SHA-256: `{PAYLOAD_SHA256}`").unwrap();
    writeln!(report, "\n## Toolchain\n\n```text\n{}\n```", metadata.rustc).unwrap();
    writeln!(
        report,
        "\n## Reproduction\n\n```powershell\n{REPRODUCE_COMMAND}\n```"
    )
    .unwrap();
    writeln!(report, "\n## Method\n").unwrap();
    writeln!(
        report,
        "The release-mode verifier enumerates every combination `0 <= a < b < c < d < e < f < g < 52` exactly once. For each combination it compares the complete raw `u16` rank returned by the public embedded-table `Evaluator` with the complete raw `u16` rank returned by the ported direct seven-card evaluator. It stops at the first mismatch."
    )
    .unwrap();

    if let Some(details) = mismatch {
        writeln!(report, "\n## First mismatch\n").unwrap();
        writeln!(report, "- Cards: `{:?}`", details.cards).unwrap();
        writeln!(report, "- Canonical key: `{}`", details.canonical_key).unwrap();
        writeln!(report, "- Formula index: `{}`", details.formula_index).unwrap();
        writeln!(report, "- Embedded rank: `{}`", details.embedded_rank).unwrap();
        writeln!(report, "- Direct rank: `{}`", details.direct_rank).unwrap();
    }
    report
}

fn write_report(path: &Path, report: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, report)?;
    Ok(())
}
