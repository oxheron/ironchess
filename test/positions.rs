// Stockfish 18 reference counts, calculated independently at every depth 1-6.
// Standard positions: https://github.com/official-stockfish/Stockfish/blob/master/tests/perft.sh
// Additional cases isolate common move-generation edge cases.
// Recheck with the verify_stockfish test in perft.rs.

perft_positions! {
    starting_position:
        "Initial position: pawn pushes, knights, and move ordering.",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        [20, 400, 8_902, 197_281, 4_865_609, 119_060_324];

    kiwipete:
        "Standard position 2: castling, pins, and dense sliding-piece attacks.",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        [48, 2_039, 97_862, 4_085_603, 193_690_690, 8_031_647_685];

    rook_pawn_endgame:
        "Standard position 3: en passant, rook checks, and pawn endgames.",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        [14, 191, 2_812, 43_238, 674_624, 11_030_083];

    promotions_and_castling:
        "Standard position 4: promotions, check evasions, and asymmetric castling rights.",
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        [6, 264, 9_467, 422_333, 15_833_292, 706_045_033];

    promotion_check_evasions:
        "Standard position 5: capture promotions, underpromotions, and check evasions.",
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        [44, 1_486, 62_379, 2_103_487, 89_941_194, 3_048_196_529];

    middlegame_pins:
        "Standard position 6: middlegame mobility and pinned pieces.",
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        [46, 2_079, 89_890, 3_894_594, 164_075_551, 6_923_051_137];

    castling_and_rook_captures:
        "Both castles and loss of castling rights when corner rooks move or are captured.",
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
        [26, 568, 13_744, 314_346, 7_594_526, 179_862_938];

    castling_through_check:
        "Kingside castling crosses an attacked f1; queenside castling remains legal.",
        "4kr2/8/8/8/8/8/8/R3K2R w KQ - 0 1",
        [23, 271, 6_617, 99_418, 2_502_908, 39_116_359];

    en_passant_white:
        "White can capture en passant immediately from the FEN.",
        "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 2",
        [7, 38, 276, 1_786, 13_207, 86_117];

    en_passant_black:
        "Black to move can capture en passant immediately from the FEN.",
        "4k3/8/8/8/3Pp3/8/8/4K3 b - d3 0 2",
        [7, 38, 276, 1_786, 13_207, 86_117];

    en_passant_horizontal_pin:
        "En passant would expose the white king to a rook along the fifth rank.",
        "8/8/8/r4pPK/8/8/8/4k3 w - f6 0 2",
        [4, 68, 316, 5_834, 32_668, 620_250];

    en_passant_diagonal_pin:
        "En passant would expose the white king to a bishop diagonal.",
        "7k/8/5b2/3pP3/8/2K5/8/8 w - d6 0 2",
        [8, 72, 558, 6_300, 43_449, 529_044];

    en_passant_captures_checker:
        "En passant captures the pawn that is checking the king.",
        "4k3/8/8/3pP3/4K3/8/8/8 w - d6 0 2",
        [8, 44, 316, 1_951, 15_351, 95_698];

    en_passant_discovered_check:
        "En passant removes both fifth-rank blockers and uncovers a rook check.",
        "8/8/8/k4PpR/8/8/8/4K3 w - g6 0 2",
        [15, 83, 1_421, 9_640, 174_072, 1_185_179];

    promotions_both_colors:
        "Both sides are one push from promotion; all four promotion pieces matter.",
        "4k3/P7/8/8/8/8/7p/4K3 w - - 0 1",
        [9, 69, 699, 7_192, 86_140, 1_016_714];

    capture_underpromotions:
        "White has both quiet and rook-capture promotions to all four pieces.",
        "1r2k3/P7/8/8/8/8/8/4K3 w - - 0 1",
        [13, 136, 1_534, 20_879, 265_449, 3_925_610];

    absolute_rook_pin:
        "A pinned rook may move along its pin ray and capture its pinner.",
        "4r1k1/8/8/8/8/8/4R3/4K3 w - - 0 1",
        [10, 129, 1_775, 25_563, 399_093, 6_165_264];

    double_check:
        "A rook and bishop give double check; only king moves can evade it.",
        "4r1k1/8/8/8/1b6/8/8/4K3 w - - 0 1",
        [3, 78, 274, 6_955, 37_340, 960_711];

    checkmate:
        "Black is checkmated; every positive perft depth has zero leaves.",
        "7k/6Q1/5K2/8/8/8/8/8 b - - 0 1",
        [0, 0, 0, 0, 0, 0];

    stalemate:
        "Black is stalemated; every positive perft depth has zero leaves.",
        "7k/5Q2/6K1/8/8/8/8/8 b - - 0 1",
        [0, 0, 0, 0, 0, 0];

    double_push_blocks_check:
        "A pawn double push blocks a rook check even though the single push does not.",
        "4k3/8/8/8/r6K/8/3P4/8 w - - 0 1",
        [5, 91, 564, 10_115, 69_596, 1_261_768];

    king_capture_defended_piece:
        "The king must not capture a knight protected by a rook.",
        "4k3/8/8/8/8/4r3/4n3/4K3 w - - 0 1",
        [4, 88, 310, 6_895, 29_976, 668_204];

}
