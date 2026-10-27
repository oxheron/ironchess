mod transposition;

use crate::dprintln;
use crate::eval::{INF, MATE, eval};
use crate::game::board::*;
use crate::game::datatypes::*;
use crate::game::state::*;
use crate::movegen::generate_moves;
use crate::movegen::moveinfo::*;
use crate::search::transposition::TranspositionEntry;
use crate::search::transposition::TranspositionTable;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

static TRANSPOSITION_TABLE: LazyLock<Mutex<TranspositionTable>> =
    LazyLock::new(|| Mutex::new(TranspositionTable::new()));

pub fn perft(depth: u32, board: &mut Board, mut state: State) -> u64 {
    // dprintln!("{:?}", depth);
    let mut count: u64 = 0;
    let moves = generate_moves(board, &state);

    if depth == 1 {
        count += moves.len() as u64;
        return count;
    }

    // dprintln!("{:?}", depth);
    for m in moves {
        // dprintln!("{:?}", m);
        state = board.do_move(state, &m);
        count += perft(depth - 1, board, state.clone());
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

    // Retain the last completed iteration if a deeper search is interrupted.
    for depth in 1..=limits.depth {
        let Some(result) = search_node(depth, board, state.clone(), -INF, INF, &mut control) else {
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

fn search_node(
    depth: u32,
    board: &mut Board,
    mut state: State,
    mut alpha: i32,
    beta: i32,
    control: &mut SearchControl<'_>,
) -> Option<NodeResult> {
    if control.interrupted() {
        return None;
    }

    let og_alpha = alpha;
    let og_beta = beta;

    let moves = generate_moves(board, &state);

    if moves.is_empty() {
        return Some(NodeResult {
            score: if generate_info(board, &state).check { -MATE } else { 0 },
            best_move: None,
        });
    }

    if state.fiftymove() >= 150 {
        return Some(NodeResult {
            score: 0,
            best_move: None,
        });
    }
    if depth == 0 {
        return Some(NodeResult {
            score: eval(board, &state),
            best_move: None,
        });
    }
    
    let entry = TRANSPOSITION_TABLE.lock().unwrap().get_check_entry(state.zobrist());
    if let Some(entry) = entry {
        if entry.depth >= depth {
            if (entry.nodetype == NodeType::EXACT) 
                || (entry.nodetype == NodeType::LOWER && entry.score >= beta)
                || (entry.nodetype == NodeType::UPPER && entry.score <= alpha) {
                return Some(NodeResult {
                    score: entry.score,
                    best_move: Some(entry.cmove),
                });
            }
        }
    }

    let mut best_score = -INF;
    let mut best_move = Move::default();

    for cmove in moves {
        state = board.do_move(state, &cmove);
        let result = search_node(depth - 1, board, state.clone(), -beta, -alpha, control);
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
        nodetype: 
            if best_score <= og_alpha {
                NodeType::UPPER
            } else if best_score >= og_beta {
                NodeType::LOWER
            } else {
                NodeType::EXACT
            }
    };

    TRANSPOSITION_TABLE.lock().unwrap().replace_entry(tt_entry);
    
    Some(NodeResult {
        score: best_score,
        best_move: Some(best_move),
    })
}
