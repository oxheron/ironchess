use super::*;
use crate::game::datatypes::PieceType;
use crate::search::start_search;

#[test]
fn position_replays_opening_moves() {
    let (board, state) = parse_position(&["startpos", "moves", "e2e4", "e7e5", "g1f3"]).unwrap();
    let (expected, _) = load_fen("rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2");
    assert_eq!(board.all(), expected.all());
    assert_eq!(board.colored(), expected.colored());
    assert_eq!(board.piece(), expected.piece());
    assert!(state.turn());
    assert_eq!(state.fiftymove(), 1);
}

#[test]
fn position_replays_castling_en_passant_and_underpromotion() {
    for (fen, notation, square, piece) in [
        (
            "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
            "e1g1",
            5,
            PieceType::ROOK,
        ),
        (
            "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 2",
            "e5d6",
            43,
            PieceType::PAWN,
        ),
        (
            "4k3/P7/8/8/8/8/8/4K3 w - - 0 1",
            "a7a8n",
            56,
            PieceType::KNIGHT,
        ),
    ] {
        let command = format!("fen {fen} moves {notation}");
        let tokens: Vec<_> = command.split_whitespace().collect();
        let (board, state) = parse_position(&tokens).unwrap();
        assert_ne!(board.all() & (1 << square), 0);
        assert_eq!(board.pieces()[square].piece_type, piece);
        assert!(state.turn());
    }
}

#[test]
fn malformed_positions_and_illegal_moves_are_rejected() {
    for command in [
        "",
        "fen",
        "startpos moves e2e5",
        "startpos moves e7e5",
        "fen 8/8/8/8/8/8/8/8 w - - 0 1",
        "fen 4k3/8/8/8/8/8/8/4K3 x - - 0 1",
        "fen 4k3/8/8/8/8/8/8/4K3 w - - 0 0",
        "fen 4k3/8/8/8/8/8/8/4K3 w - z9 0 1",
    ] {
        assert!(
            parse_position(&command.split_whitespace().collect::<Vec<_>>()).is_err(),
            "{command}"
        );
    }
}

fn best_move(fen: &str, depth: u32) -> String {
    let (mut board, state) = load_fen(fen);
    let before = board.clone();
    let best = start_search(
        SearchLimits {
            depth,
            movetime: None,
            nodes: None,
        },
        &mut board,
        state,
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(board.all(), before.all());
    assert_eq!(board.colored(), before.colored());
    assert_eq!(board.piece(), before.piece());
    best.to_string()
}

#[test]
fn search_captures_material_even_late_in_a_game() {
    assert_eq!(best_move("3q3k/8/8/8/8/8/8/3R3K w - - 0 100", 2), "d1d8");
}

#[test]
fn search_finds_mate_at_the_depth_boundary() {
    assert_eq!(best_move("7k/8/5KQ1/8/8/8/8/8 w - - 0 1", 1), "g6g7");
    let (mut board, state) = load_fen("7k/6Q1/5K2/8/8/8/8/8 b - - 0 1");
    assert!(
        start_search(
            SearchLimits {
                depth: 1,
                movetime: None,
                nodes: None,
            },
            &mut board,
            state,
            &AtomicBool::new(false),
        )
        .is_none()
    );
}

#[test]
fn interrupted_search_restores_the_board() {
    for (stop, nodes, movetime) in [
        (true, None, None),
        (false, Some(100), None),
        (false, None, Some(Duration::ZERO)),
    ] {
        let (mut board, state) = load_fen(START_FEN);
        let before = board.clone();
        let legal = generate_moves(&board, &state);
        let best = start_search(
            SearchLimits {
                depth: 64,
                movetime,
                nodes,
            },
            &mut board,
            state.clone(),
            &AtomicBool::new(stop),
        );
        if stop || movetime == Some(Duration::ZERO) {
            assert!(best.is_none());
        } else {
            assert!(legal.contains(&best.unwrap()));
        }
        assert_eq!(board.all(), before.all());
        assert_eq!(board.colored(), before.colored());
        assert_eq!(board.piece(), before.piece());
        assert_eq!(generate_moves(&board, &state), legal);
    }
}

#[test]
fn clock_limits_use_the_side_to_move_and_keep_a_time_reserve() {
    let tokens = ["wtime", "1000", "btime", "90000", "winc", "60"];
    assert_eq!(
        parse_limits(&tokens, false).movetime,
        Some(Duration::from_millis(63))
    );
    assert_eq!(
        parse_limits(&tokens, true).movetime,
        Some(Duration::from_millis(3000))
    );
    assert_eq!(
        parse_limits(&["wtime", "0"], false).movetime,
        Some(Duration::from_millis(1))
    );
    assert_eq!(parse_limits(&[], false).depth, 3);
}

#[test]
fn search_reports_completed_depths_with_gui_readable_moves_and_scores() {
    let (mut board, state) = load_fen("3q3k/8/8/8/8/8/8/3R3K w - - 0 100");
    let legal = generate_moves(&board, &state);
    let mut reports = Vec::new();
    let best = start_search_with_info(
        SearchLimits {
            depth: 3,
            movetime: None,
            nodes: None,
        },
        &mut board,
        state,
        &AtomicBool::new(false),
        |info| reports.push(info),
    )
    .unwrap();

    assert_eq!(
        reports.iter().map(|info| info.depth).collect::<Vec<_>>(),
        [1, 2, 3]
    );
    assert!(reports.windows(2).all(|pair| pair[0].nodes < pair[1].nodes));
    assert!(
        reports
            .windows(2)
            .all(|pair| pair[0].elapsed <= pair[1].elapsed)
    );
    for info in &reports {
        assert!(legal.contains(&info.best_move));
        assert!(info.nodes > 0);
        assert_eq!(info.score, 500);
        let message = format_info(*info);
        assert!(message.starts_with(&format!(
            "info depth {} multipv 1 score cp 500 nodes ",
            info.depth
        )));
        assert!(message.ends_with(" pv d1d8"));
    }
    assert_eq!(reports.last().unwrap().best_move, best);
}

#[test]
fn search_returns_no_move_or_reports_when_no_iteration_completes() {
    let (mut board, state) = load_fen(START_FEN);
    let mut reports = Vec::new();
    let best = start_search_with_info(
        SearchLimits {
            depth: 64,
            movetime: None,
            nodes: Some(0),
        },
        &mut board,
        state,
        &AtomicBool::new(false),
        |info| reports.push(info),
    );
    assert!(best.is_none());
    assert!(reports.is_empty());
}
