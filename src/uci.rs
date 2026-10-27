//! Minimal UCI interface: https://backscattering.de/chess/uci/
//! Run `cargo run --release`, or point a GUI at `target/release/ironchess`.

use std::io::{self, BufRead, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::game::{board::Board, load::load_fen, state::State};
use crate::movegen::generate_moves;
use crate::search::{SearchInfo, SearchLimits, start_search_with_info};

const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
type SearchJob = (Arc<AtomicBool>, JoinHandle<()>);

pub fn run() -> io::Result<()> {
    let (mut board, mut state) = load_fen(START_FEN);
    let mut job: Option<SearchJob> = None;

    for line in io::stdin().lock().lines() {
        let line = line?;
        let tokens: Vec<&str> = line.split_whitespace().collect();
        let Some(command) = tokens.first() else {
            continue;
        };
        match *command {
            "uci" => reply(concat!(
                "id name Ironchess ",
                env!("CARGO_PKG_VERSION"),
                "\nid author Ironchess contributors\nuciok"
            ))?,
            "isready" => reply("readyok")?,
            "ucinewgame" => {
                stop_search(&mut job);
                (board, state) = load_fen(START_FEN);
            }
            "position" => {
                stop_search(&mut job);
                match parse_position(&tokens[1..]) {
                    Ok(position) => (board, state) = position,
                    Err(error) => reply(&format!("info string {error}"))?,
                }
            }
            "go" => {
                stop_search(&mut job);
                let limits = parse_limits(&tokens[1..], state.turn());
                let infinite = tokens.contains(&"infinite");
                let mut search_board = board.clone();
                let search_state = state.clone();
                let stop = Arc::new(AtomicBool::new(false));
                let search_stop = Arc::clone(&stop);
                let worker = thread::spawn(move || {
                    let best = start_search_with_info(
                        limits,
                        &mut search_board,
                        search_state,
                        &search_stop,
                        |info| {
                            if let Err(error) = reply(&format_info(info)) {
                                eprintln!("UCI output error: {error}");
                                search_stop.store(true, Ordering::Relaxed);
                            }
                        },
                    );
                    // Even terminal positions in infinite mode wait for stop.
                    while infinite && !search_stop.load(Ordering::Relaxed) {
                        thread::sleep(Duration::from_millis(5));
                    }
                    let notation =
                        best.map_or_else(|| "0000".to_owned(), |cmove| cmove.to_string());
                    if let Err(error) = reply(&format!("bestmove {notation}")) {
                        eprintln!("UCI output error: {error}");
                    }
                });
                job = Some((stop, worker));
            }
            "stop" => stop_search(&mut job),
            "quit" => break,
            // No configurable options or pondering are advertised.
            _ => {}
        }
    }
    stop_search(&mut job);
    Ok(())
}

fn reply(message: &str) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{message}")?;
    stdout.flush()
}

fn format_info(info: SearchInfo) -> String {
    let milliseconds = info.elapsed.as_millis();
    let nps = u128::from(info.nodes) * 1000 / milliseconds.max(1);
    // En Croissant reads its evaluation and candidate move from info/PV.
    // Only the root move is reported; no move ordering is applied.
    format!(
        "info depth {} multipv 1 score cp {} nodes {} time {} nps {} pv {}",
        info.depth, info.score, info.nodes, milliseconds, nps, info.best_move
    )
}

fn stop_search(job: &mut Option<SearchJob>) {
    if let Some((stop, worker)) = job.take() {
        stop.store(true, Ordering::Relaxed);
        if worker.join().is_err() {
            eprintln!("Search thread panicked");
        }
    }
}

fn parse_position(tokens: &[&str]) -> Result<(Board, State), &'static str> {
    let (fen, rest) = match tokens.first().copied() {
        Some("startpos") => (START_FEN.to_owned(), &tokens[1..]),
        Some("fen") if tokens.len() >= 7 && valid_fen(&tokens[1..7]) => {
            (tokens[1..7].join(" "), &tokens[7..])
        }
        _ => return Err("invalid position; expected startpos or a six-field FEN"),
    };
    if !rest.is_empty() && rest[0] != "moves" {
        return Err("expected moves after the position");
    }
    let (mut board, mut state) = load_fen(&fen);
    for notation in rest.iter().skip(1) {
        let cmove = generate_moves(&board, &state)
            .into_iter()
            .find(|cmove| cmove.to_string() == *notation)
            .ok_or("illegal move in position command")?;
        state = board.do_move(state, &cmove);
    }
    Ok((board, state))
}

// Validate the shape and numeric fields before calling the existing FEN loader.
// Position commands from a chess GUI are expected to describe legal games.
fn valid_fen(fields: &[&str]) -> bool {
    let ranks: Vec<&str> = fields[0].split('/').collect();
    let mut kings = [0, 0];
    if ranks.len() != 8 {
        return false;
    }
    for rank in ranks {
        let mut width = 0;
        for piece in rank.bytes() {
            width += match piece {
                b'1'..=b'8' => piece - b'0',
                b'K' => {
                    kings[0] += 1;
                    1
                }
                b'k' => {
                    kings[1] += 1;
                    1
                }
                b'p' | b'n' | b'b' | b'r' | b'q' | b'P' | b'N' | b'B' | b'R' | b'Q' => 1,
                _ => return false,
            };
            if width > 8 {
                return false;
            }
        }
        if width != 8 {
            return false;
        }
    }
    let ep = fields[3].as_bytes();
    kings == [1, 1]
        && matches!(fields[1], "w" | "b")
        && (fields[2] == "-" || fields[2].bytes().all(|right| b"KQkq".contains(&right)))
        && (fields[3] == "-"
            || (ep.len() == 2
                && (b'a'..=b'h').contains(&ep[0])
                && ep[1] == if fields[1] == "w" { b'6' } else { b'3' }))
        && fields[4].parse::<u16>().is_ok_and(|n| n <= u16::MAX - 128)
        && fields[5]
            .parse::<u32>()
            .is_ok_and(|n| n > 0 && n <= u32::MAX / 2 - 128)
}

fn parse_limits(tokens: &[&str], black: bool) -> SearchLimits {
    let number = |name| {
        tokens
            .windows(2)
            .find(|pair| pair[0] == name)
            .and_then(|pair| pair[1].parse::<u64>().ok())
    };
    let infinite = tokens.contains(&"infinite");
    let movetime = if infinite {
        None
    } else if let Some(time) = number("movetime") {
        Some(time.saturating_sub(5).max(1))
    } else {
        number(if black { "btime" } else { "wtime" }).map(|remaining| {
            let increment = number(if black { "binc" } else { "winc" }).unwrap_or(0);
            let moves = number("movestogo").unwrap_or(30).max(1);
            (remaining / moves)
                .saturating_add(increment / 2)
                .min(remaining.saturating_sub(20))
                .max(1)
        })
    }
    .map(Duration::from_millis);
    let nodes = number("nodes");
    let depth = number("depth")
        .map(|depth| depth.clamp(1, 64) as u32)
        .unwrap_or(if infinite || movetime.is_some() || nodes.is_some() {
            64
        } else {
            3
        });
    SearchLimits {
        depth,
        movetime,
        nodes,
    }
}

#[cfg(test)]
#[path = "../test/uci.rs"]
mod tests;
