use super::{generate_ordered_moves, memory::SearchFrame, order_moves};
use crate::game::load::load_fen;
use crate::movegen::{generate_info, generate_move_bitset};
use crate::test_support::{capture_moves, legal_moves};
use std::collections::HashSet;

#[test]
fn ordering_prioritizes_tt_then_captures_and_preserves_remaining_moves() {
    let (board, state) = load_fen("3q3k/8/8/8/4p3/8/8/1B1R3K w - - 0 1");
    let legal = legal_moves(&board, &state);

    // A quiet TT move leads captures; a capture TT move is emitted only once.
    for (tt_name, expected_prefix) in [
        (None, vec!["d1d8", "b1e4"]),
        (Some("h1g1"), vec!["h1g1", "d1d8", "b1e4"]),
        (Some("b1e4"), vec!["b1e4", "d1d8"]),
    ] {
        let tt_move = tt_name.map(|name| *legal.iter().find(|m| m.to_string() == name).unwrap());
        let mut frame = SearchFrame::default();
        generate_move_bitset(&board, &state, &generate_info(&board, &state), &mut frame);
        assert!(order_moves(&board, &state, &mut frame, tt_move));
        let names: Vec<_> = frame
            .ordered_moves
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(&names[..expected_prefix.len()], expected_prefix);

        let expected_remainder: Vec<_> = legal
            .iter()
            .map(ToString::to_string)
            .filter(|name| !expected_prefix.contains(&name.as_str()))
            .collect();
        assert_eq!(&names[expected_prefix.len()..], expected_remainder);
    }
}

#[test]
fn ordering_rejects_illegal_capture_and_tt_candidates() {
    for (fen, illegal_name) in [
        // Moving the rook sideways exposes its king to the rook on e8.
        ("4r1k1/8/8/8/8/3pR3/8/4K3 w - - 0 1", "e3d3"),
        // En passant would expose the king to the rook on a5.
        ("8/8/8/r4pPK/8/8/8/4k3 w - f6 0 2", "g5f6"),
    ] {
        let (board, state) = load_fen(fen);
        let captures = capture_moves(&board, &state);
        let illegal = *captures
            .iter()
            .find(|m| m.to_string() == illegal_name)
            .unwrap();
        let legal = legal_moves(&board, &state);
        assert!(!legal.contains(&illegal), "{fen}");

        let mut frame = SearchFrame::default();
        generate_move_bitset(&board, &state, &generate_info(&board, &state), &mut frame);
        assert!(order_moves(&board, &state, &mut frame, Some(illegal)));
        let mut remaining: HashSet<_> = legal.into_iter().collect();
        for m in frame.ordered_moves {
            assert!(remaining.remove(&m), "illegal or repeated move: {m}");
        }
        assert!(remaining.is_empty(), "a legal move was omitted: {fen}");
    }
}

#[test]
fn tt_promotions_preserve_all_four_choices_for_both_colors() {
    for (fen, promotions, tt_choices) in [
        (
            "1r2k3/P7/8/8/8/8/8/4K3 w - - 0 1",
            [
                "a7a8q", "a7b8q", "a7a8n", "a7b8n", "a7a8r", "a7b8r", "a7a8b", "a7b8b",
            ],
            ["a7b8n", "a7a8b"],
        ),
        (
            "4k3/8/8/8/8/8/p7/1R2K3 b - - 0 1",
            [
                "a2a1q", "a2b1q", "a2a1n", "a2b1n", "a2a1r", "a2b1r", "a2a1b", "a2b1b",
            ],
            ["a2b1n", "a2a1b"],
        ),
    ] {
        let (board, state) = load_fen(fen);
        let legal = legal_moves(&board, &state);
        for tt_name in [None, Some(tt_choices[0]), Some(tt_choices[1])] {
            let tt_move =
                tt_name.map(|name| *legal.iter().find(|m| m.to_string() == name).unwrap());
            let mut frame = SearchFrame::default();
            generate_move_bitset(&board, &state, &generate_info(&board, &state), &mut frame);
            assert!(order_moves(&board, &state, &mut frame, tt_move));

            let names: Vec<_> = frame
                .ordered_moves
                .iter()
                .map(ToString::to_string)
                .collect();
            let expected: Vec<_> = tt_name
                .into_iter()
                .chain(promotions.into_iter().filter(|name| Some(*name) != tt_name))
                .collect();
            assert_eq!(&names[..expected.len()], expected, "{fen}");
            assert_eq!(frame.ordered_moves.len(), legal.len());
            assert_eq!(
                frame.ordered_moves.iter().copied().collect::<HashSet<_>>(),
                legal.iter().copied().collect::<HashSet<_>>()
            );
        }
    }
}

#[test]
fn frame_reuse_clears_old_moves_and_retains_buffer_capacity() {
    let mut frame = SearchFrame::default();
    let cases = [
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            20,
        ),
        ("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 2", 7),
        ("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1", 0),
        ("4k3/8/8/8/3Pp3/8/8/4K3 b - d3 0 2", 7),
        ("1r2k3/P7/8/8/8/8/8/4K3 w - - 0 1", 13),
    ];
    let (board, state) = load_fen(cases[0].0);
    generate_ordered_moves(&board, &state, &mut frame);
    let allocation = frame.ordered_moves.as_ptr();

    for (fen, expected_count) in cases {
        let (board, state) = load_fen(fen);
        generate_ordered_moves(&board, &state, &mut frame);
        assert_eq!(frame.ordered_moves.len(), expected_count, "{fen}");
        assert_eq!(frame.ordered_moves.as_ptr(), allocation);
        assert!(frame.move_bitset.is_empty());
    }
}
