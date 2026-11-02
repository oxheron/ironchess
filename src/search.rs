pub mod memory;
mod transposition;

use crate::eval::{INF, MATE, eval};
use crate::game::board::*;
use crate::game::datatypes::*;
use crate::game::state::*;
use crate::movegen::captures::generate_mvv_lva_bitboards;
use crate::movegen::generate_move_bitset;
use crate::movegen::moveinfo::*;
use crate::search::memory::SearchFrame;
use crate::search::transposition::TranspositionEntry;
use crate::search::transposition::TranspositionTable;
use crate::utils::{RANK_2, RANK_7};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

static TRANSPOSITION_TABLE: LazyLock<Mutex<TranspositionTable>> =
    LazyLock::new(|| Mutex::new(TranspositionTable::new()));

pub fn perft(depth: u32, board: &mut Board, state: State) -> u64 {
    let mut frames: Vec<_> = (0..depth).map(|_| SearchFrame::default()).collect();
    perft_node(depth, board, state, &mut frames)
}

fn perft_node(depth: u32, board: &mut Board, mut state: State, frames: &mut [SearchFrame]) -> u64 {
    if depth == 0 {
        return 1;
    }

    let (frame, remaining) = frames.split_first_mut().expect("missing perft frame");
    generate_ordered_moves(board, &state, frame);

    if depth == 1 {
        return frame.ordered_moves.len() as u64;
    }

    let mut count = 0;
    for index in 0..frame.ordered_moves.len() {
        let m = frame.ordered_moves[index];
        state = board.do_move(state, &m);
        count += perft_node(depth - 1, board, state.clone(), remaining);
        state = board.do_unmove(state, &m);
    }

    count
}

pub struct SearchLimits {
    pub depth: u32,
    pub movetime: Option<Duration>,
    pub nodes: Option<u64>,
}

#[derive(Clone, Copy)]
pub struct SearchInfo {
    pub depth: u32,
    pub score: i32,
    pub nodes: u64,
    pub elapsed: Duration,
    pub best_move: Move,
}

struct NodeResult {
    score: i32,
    best_move: Option<Move>,
}

struct SearchControl<'a> {
    stop: &'a AtomicBool,
    deadline: Option<Instant>,
    nodes_left: Option<u64>,
    nodes: u64,
}

impl SearchControl<'_> {
    fn interrupted(&mut self) -> bool {
        if self.stop.load(Ordering::Relaxed)
            || self
                .deadline
                .is_some_and(|deadline| Instant::now() >= deadline)
            || self.nodes_left == Some(0)
        {
            return true;
        }
        if let Some(nodes) = &mut self.nodes_left {
            *nodes -= 1;
        }
        self.nodes += 1;
        false
    }
}

pub fn start_search(
    limits: SearchLimits,
    board: &mut Board,
    state: State,
    stop: &AtomicBool,
) -> Option<Move> {
    start_search_with_info(limits, board, state, stop, |_| {})
}

pub fn start_search_with_info(
    limits: SearchLimits,
    board: &mut Board,
    state: State,
    stop: &AtomicBool,
    mut report: impl FnMut(SearchInfo),
) -> Option<Move> {
    let mut best_move = None;
    let now = Instant::now();
    let mut control = SearchControl {
        stop,
        deadline: limits
            .movetime
            .map(|time| now.checked_add(time).unwrap_or(now)),
        nodes_left: limits.nodes,
        nodes: 0,
    };

    let mut frames: Vec<SearchFrame> = Vec::new();

    // Retain the last completed iteration if a deeper search is interrupted.
    for depth in 1..=limits.depth {
        // Leaves also need a frame for mate/stalemate detection.
        frames.resize_with(depth as usize + 1, SearchFrame::default);

        let Some(result) = search_node(
            depth,
            board,
            state.clone(),
            -INF,
            INF,
            &mut control,
            &mut frames,
        ) else {
            break;
        };

        let Some(iteration_best) = result.best_move else {
            break;
        };

        best_move = Some(iteration_best);
        report(SearchInfo {
            depth,
            score: result.score,
            nodes: control.nodes,
            elapsed: now.elapsed(),
            best_move: iteration_best,
        });
    }
    best_move
}

pub(crate) fn generate_ordered_moves(board: &Board, state: &State, frame: &mut SearchFrame) {
    let info = generate_info(board, state);
    generate_move_bitset(board, state, &info, frame);
    order_moves(board, state, frame, None);
}

fn order_moves(
    board: &Board,
    state: &State,
    search_frame: &mut SearchFrame,
    entry: Option<Move>,
) -> bool {
    search_frame.ordered_moves.clear();
    let promoting_pawns =
        board.bitboard(PieceType::PAWN, state.turn()) & if state.turn() { RANK_2 } else { RANK_7 };

    // Keep a TT promotion's destination available for the other three choices.
    let tt_promotion = entry.filter(|m| {
        matches!(
            m.piece,
            PieceType::QUEEN | PieceType::KNIGHT | PieceType::ROOK | PieceType::BISHOP
        ) && (promoting_pawns & (1u64 << m.start)) != 0
            && search_frame.move_bitset.check_move(m)
    });

    // Previous best TT move first.
    if let Some(tt_move) = entry {
        if tt_promotion == Some(tt_move) {
            search_frame.ordered_moves.push(tt_move);
        } else if tt_move.piece == PieceType::EMPTY
            && (promoting_pawns & (1u64 << tt_move.start)) == 0
            && search_frame.move_bitset.clear_move(&tt_move)
        {
            search_frame.ordered_moves.push(tt_move);
        }
    }

    // Expand every legal promotion destination in Q, N, R, B order.
    for promotion in [
        PieceType::QUEEN,
        PieceType::KNIGHT,
        PieceType::ROOK,
        PieceType::BISHOP,
    ] {
        let mut pawns = promoting_pawns;
        while pawns != 0 {
            let start = pawns.trailing_zeros() as u8;
            let mut destinations = search_frame.move_bitset.get_square(start);

            while destinations != 0 {
                let end = destinations.trailing_zeros() as u8;
                let cmove = Move {
                    start,
                    end,
                    piece: promotion,
                };

                if Some(cmove) != tt_promotion {
                    search_frame.ordered_moves.push(cmove);
                }

                destinations &= destinations - 1;
            }

            pawns &= pawns - 1;
        }
    }

    // Consume destinations only after all four promotion choices are emitted.
    let mut pawns = promoting_pawns;
    while pawns != 0 {
        let start = pawns.trailing_zeros() as u8;
        search_frame.move_bitset.set_square(start, 0);
        pawns &= pawns - 1;
    }

    // Only legal captures are searched, in MVV-LVA order.
    let captures = generate_mvv_lva_bitboards(board, state);

    for (piece, attacks_bb) in captures {
        let mut attacks_bb = attacks_bb;
        while attacks_bb != 0 {
            let end = attacks_bb.trailing_zeros() as u8;
            let cmove = Move {
                start: piece,
                end: end,
                piece: PieceType::EMPTY,
            };

            if search_frame.move_bitset.clear_move(&cmove) {
                search_frame.ordered_moves.push(cmove);
            }

            attacks_bb &= attacks_bb - 1;
        }
    }

    let mut pieces_iter: u64 = board.colored()[state.turn() as usize];
    while pieces_iter != 0 {
        let square = pieces_iter.trailing_zeros() as u8;
        let mut ends_bb = search_frame.move_bitset.get_square(square);

        while ends_bb != 0 {
            let end = ends_bb.trailing_zeros() as u8;
            let cmove = Move {
                start: square,
                end: end,
                piece: PieceType::EMPTY,
            };

            search_frame.ordered_moves.push(cmove);

            ends_bb &= ends_bb - 1;
        }

        pieces_iter &= pieces_iter - 1;
    }

    search_frame.move_bitset.clear();

    true
}

fn search_node(
    depth: u32,
    board: &mut Board,
    mut state: State,
    mut alpha: i32,
    beta: i32,
    control: &mut SearchControl<'_>,
    frames: &mut [SearchFrame],
) -> Option<NodeResult> {
    if control.interrupted() {
        return None;
    }

    let Some((frame, remaining)) = frames.split_first_mut() else {
        return None;
    };

    let og_alpha = alpha;
    let og_beta = beta;

    let info = generate_info(board, &state);
    if !generate_move_bitset(board, &state, &info, frame) {
        panic!("Failed to generate moves");
    }

    if frame.move_bitset.is_empty() {
        return Some(NodeResult {
            score: if info.check { -MATE } else { 0 },
            best_move: None,
        });
    }

    if depth == 0 || state.fiftymove() >= 150 {
        let score = if state.fiftymove() >= 150 {
            0
        } else {
            eval(board, &state)
        };
        return Some(NodeResult {
            score,
            best_move: None,
        });
    }

    let entry = TRANSPOSITION_TABLE
        .lock()
        .unwrap()
        .get_check_entry(state.zobrist());
    if let Some(entry) = entry {
        if entry.depth >= depth {
            if (entry.nodetype == NodeType::EXACT)
                || (entry.nodetype == NodeType::LOWER && entry.score >= beta)
                || (entry.nodetype == NodeType::UPPER && entry.score <= alpha)
            {
                return Some(NodeResult {
                    score: entry.score,
                    best_move: Some(entry.cmove),
                });
            }
        }
    }

    if !order_moves(board, &state, frame, entry.map(|entry| entry.cmove)) {
        return None;
    }

    let mut best_score = -INF;
    let mut best_move = Move::default();

    for index in 0..frame.ordered_moves.len() {
        let cmove = frame.ordered_moves[index];
        state = board.do_move(state, &cmove);
        let result = search_node(
            depth - 1,
            board,
            state.clone(),
            -beta,
            -alpha,
            control,
            remaining,
        );
        // Always undo before propagating cancellation to the caller.
        state = board.do_unmove(state, &cmove);
        let score = -result?.score;
        alpha = alpha.max(score);

        if score > best_score {
            best_score = score;
            best_move = cmove;
        }

        if alpha >= beta {
            break;
        }
    }

    let tt_entry = TranspositionEntry {
        zobrist: state.zobrist(),
        score: best_score,
        depth: depth,
        cmove: best_move,
        nodetype: if best_score <= og_alpha {
            NodeType::UPPER
        } else if best_score >= og_beta {
            NodeType::LOWER
        } else {
            NodeType::EXACT
        },
    };

    TRANSPOSITION_TABLE.lock().unwrap().replace_entry(tt_entry);

    Some(NodeResult {
        score: best_score,
        best_move: Some(best_move),
    })
}

#[cfg(test)]
#[path = "../test/search.rs"]
mod tests;
