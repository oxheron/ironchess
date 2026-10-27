//! FEN/perft regression tests against Stockfish reference counts.
//!
//! Depth means plies (individual moves by either side), so depth 6 is three
//! moves by each player. Perft counts legal move sequences, without stopping
//! for repetition, the fifty-move rule, or insufficient material.
//!
//! Run depths 1-4 for all positions:
//!     cargo test --release
//!
//! Run only depths 1-3:
//!     cargo test --release -- --skip depth_4
//!
//! Run all engine tests through depth 6 (billions of leaves; can take a while):
//!     cargo test --release -- --include-ignored --skip verify_stockfish
//!
//! Run one position, including its deep tests:
//!     cargo test --release rook_pawn_endgame:: -- --include-ignored
//!
//! Independently recheck every stored count using Stockfish, without Ironchess:
//!     cargo test --release verify_stockfish -- --ignored --nocapture
//!
//! Stockfish must be on PATH for that last command, or STOCKFISH may specify
//! its executable path. Normal regression tests do not need Stockfish.

use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Instant;

use crate::game::load::load_fen;
use crate::search::perft;

struct PerftCase {
    name: &'static str,
    description: &'static str,
    fen: &'static str,
    nodes: [u64; 6],
}

fn assert_perft(case: &PerftCase, depth: u32) {
    let (mut board, state) = load_fen(case.fen);
    let started = Instant::now();
    // Exercise the engine's recursive search and its make/unmake path.
    let actual = perft(depth, &mut board, state);
    let expected = case.nodes[depth as usize - 1];
    eprintln!(
        "{} depth {depth}: {actual} nodes ({:.3}s)",
        case.name,
        started.elapsed().as_secs_f64()
    );
    assert_eq!(
        actual, expected,
        "{}: {}\nFEN: {}\nDepth: {depth} plies\nReproduce in Stockfish:\nposition fen {}\ngo perft {depth}",
        case.name, case.description, case.fen, case.fen
    );
}

macro_rules! perft_depths {
    () => {
        #[test]
        fn depth_1() {
            super::assert_perft(&CASE, 1);
        }

        #[test]
        fn depth_2() {
            super::assert_perft(&CASE, 2);
        }

        #[test]
        fn depth_3() {
            super::assert_perft(&CASE, 3);
        }

        #[test]
        fn depth_4() {
            super::assert_perft(&CASE, 4);
        }

        #[test]
        #[ignore = "expensive perft; run with --release and --include-ignored"]
        fn depth_5() {
            super::assert_perft(&CASE, 5);
        }

        #[test]
        #[ignore = "expensive perft; run with --release and --include-ignored"]
        fn depth_6() {
            super::assert_perft(&CASE, 6);
        }
    };
}

// Each entry creates six independently runnable tests and is also made
// available to the optional Stockfish verifier below.
macro_rules! perft_positions {
    ($($name:ident: $description:literal, $fen:literal, $nodes:expr;)+) => {
        const CASES: &[PerftCase] = &[
            $(PerftCase {
                name: stringify!($name),
                description: $description,
                fen: $fen,
                nodes: $nodes,
            },)+
        ];

        $(mod $name {
            const CASE: super::PerftCase = super::PerftCase {
                name: stringify!($name),
                description: $description,
                fen: $fen,
                nodes: $nodes,
            };

            perft_depths!();
        })+
    };
}

include!("positions.rs");

#[test]
#[ignore = "requires Stockfish; independently verifies all fixtures through depth 6"]
fn verify_stockfish() {
    let executable = std::env::var_os("STOCKFISH").unwrap_or_else(|| "stockfish".into());

    for case in CASES {
        // Keep one Stockfish process per FEN, asking it to calculate each depth
        // separately. Commands are processed in order; perft runs synchronously.
        let mut child = Command::new(&executable)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|error| {
                panic!(
                    "Cannot start {}: {error}. Install Stockfish or set STOCKFISH to its path.",
                    executable.to_string_lossy()
                )
            });

        {
            let mut stdin = child.stdin.take().expect("Stockfish stdin");
            writeln!(stdin, "uci\nisready").expect("write UCI handshake");
            for depth in 1..=6 {
                writeln!(stdin, "position fen {}\ngo perft {depth}", case.fen)
                    .expect("write perft request");
            }
            writeln!(stdin, "quit").expect("write Stockfish quit");
        }

        let output = child.wait_with_output().expect("wait for Stockfish");
        assert!(
            output.status.success(),
            "Stockfish failed for {}: {}",
            case.name,
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).expect("Stockfish UTF-8 output");
        let version = stdout
            .lines()
            .find_map(|line| line.strip_prefix("id name "))
            .filter(|name| name.contains("Stockfish"))
            .expect("expected a Stockfish UCI identity");
        assert!(stdout.contains("uciok") && stdout.contains("readyok"));

        let counts: Vec<u64> = stdout
            .lines()
            .filter_map(|line| line.strip_prefix("Nodes searched:"))
            .map(|count| count.trim().parse().expect("Stockfish perft node count"))
            .collect();
        assert_eq!(
            counts.len(),
            6,
            "Expected six perft results for {}:\n{stdout}",
            case.name
        );
        for (index, actual) in counts.iter().enumerate() {
            assert_eq!(
                *actual,
                case.nodes[index],
                "{} fixture differs from {version} at depth {}\nFEN: {}",
                case.name,
                index + 1,
                case.fen
            );
        }
        eprintln!("{version} confirmed {} depths 1-6: {counts:?}", case.name);
    }
}
