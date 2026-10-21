pub mod magic;
pub mod moveinfo;

use crate::dprintln;
use crate::game::{board::Board, datatypes::*, state::State};
use crate::utils::*;

pub use crate::movegen::magic::*;
pub use crate::movegen::moveinfo::*;

// Also generates moveinfo
pub fn generate_moves(board: &Board, state: &State) -> Vec<Move> {
    let moveinfo = generate_info(board, state);
    let move_bb = generate_move_bitboards(board, state, &moveinfo);
    extract_moves(move_bb, board)
}

pub fn extract_moves(moves: Vec<(u8, u64)>, board: &Board) -> Vec<Move> {
    let mut moves_list: Vec<Move> = Vec::new();

    for (sq, m) in moves {
        let mut end_iter: u64 = m;
        while end_iter != 0 {
            let esq = end_iter.trailing_zeros() as u8;

            if rank(esq) % 7 == 0 && board.pieces()[sq as usize].piece_type == PieceType::PAWN {
                moves_list.push(Move {
                    start: sq,
                    end: esq,
                    piece: PieceType::ROOK,
                });
                moves_list.push(Move {
                    start: sq,
                    end: esq,
                    piece: PieceType::BISHOP,
                });
                moves_list.push(Move {
                    start: sq,
                    end: esq,
                    piece: PieceType::QUEEN,
                });
                moves_list.push(Move {
                    start: sq,
                    end: esq,
                    piece: PieceType::KNIGHT,
                });
            } else {
                moves_list.push(Move {
                    start: sq,
                    end: esq,
                    piece: PieceType::EMPTY,
                });
            }

            end_iter &= end_iter - 1;
        }
    }

    moves_list
}

// Returns a list of pieces and associated legal move bitboards
pub fn generate_move_bitboards(
    board: &Board,
    state: &State,
    moveinfo: &MoveInfo,
) -> Vec<(u8, u64)> {
    let mut square_move_list: Vec<(u8, u64)> = Vec::new();

    let mut piece_iter = board.colored()[state.turn() as usize];
    while piece_iter != 0 {
        let square = piece_iter.trailing_zeros() as u8;
        let piece = board.pieces()[square as usize];

        // If check, pins cannot help
        // Logic goes double check only kings can help
        // Single check, make sure to check against blocking or capture squares, don't need to worry about EP case
        // Also in single check, pins cannot have any legal moves. so ideally the logic is

        if piece.piece_type == PieceType::KING {
            let mut moves = generate_king(square, board, state, moveinfo);
            moves |= generate_castles(board, state, moveinfo);
            square_move_list.push((square, moves));
            piece_iter &= piece_iter - 1;
            continue;
        }

        // Only king can generate moves in double check
        if moveinfo.double_check {
            piece_iter &= piece_iter - 1;
            continue;
        }

        if moveinfo.check && ((1u64 << square) & moveinfo.pins) > 0 {
            piece_iter &= piece_iter - 1;
            continue;
        }

        let mut moves = generate_nonking(square, piece, board, state);

        if moveinfo.check {
            moves &= (1u64 << moveinfo.check_square) | moveinfo.stop_check_squares;
        }

        if state.capture_type() == (PieceType::EP as u8) && piece.piece_type == PieceType::PAWN {
            moves |= generate_ep(square, board, state, moveinfo);
        }

        if ((1u64 << square) & moveinfo.pins) > 0 {
            moves &= (moveinfo.pin_rays 
                & LINE[square as usize * 64 + board.bitboard(PieceType::KING, state.turn()).trailing_zeros() as usize]);
        }

        square_move_list.push((square, moves));
        piece_iter &= piece_iter - 1;
    }

    square_move_list
}

pub fn find_king_attacks(bb: u64) -> u64 {
    (bb & NOT_FILE_H) << 1          // east
        | (bb & NOT_FILE_A) >> 1                  // west
        | bb << 8                            // north
        | bb >> 8                            // south
        | (bb & NOT_FILE_H) << 9                  // north-east
        | (bb & NOT_FILE_A) << 7                  // north-west
        | (bb & NOT_FILE_H) >> 7                  // south-east
        | (bb & NOT_FILE_A) >> 9 // south-west
}

fn generate_king(square: u8, board: &Board, state: &State, moveinfo: &MoveInfo) -> u64 {
    let k = 1u64 << square;

    find_king_attacks(k) & !board.colored()[state.turn() as usize] & !moveinfo.attacked_squares
}

fn generate_nonking(square: u8, piece: Piece, board: &Board, state: &State) -> u64 {
    match piece.piece_type {
        PieceType::PAWN => generate_pawn(square, board, state),
        PieceType::KNIGHT => generate_knight(square, board, state),
        PieceType::BISHOP => generate_sliding(square, board, state, true),
        PieceType::ROOK => generate_sliding(square, board, state, false),
        PieceType::QUEEN => {
            generate_sliding(square, board, state, true)
                | generate_sliding(square, board, state, false)
        }
        _ => {
            board.diagnose();
            dprintln!("{:?}", board.pieces()[square as usize]);
            println!("{}", square);
            panic!("Invalid piece in generate non king");
        }
    }
}

fn generate_sliding(square: u8, board: &Board, state: &State, diag: bool) -> u64 {
    generate_sliding_occupancy(square, board.all(), diag) & !board.colored()[state.turn() as usize]
}

pub fn generate_sliding_occupancy(square: u8, occ: u64, diag: bool) -> u64 {
    let m = MAGICATTACKS.0[square as usize + (diag as usize) * 64];

    let blockers = occ & m.mask;

    let index = (blockers.wrapping_mul(m.magic)) >> m.shift;

    MAGICATTACKS.1[m.offset + index as usize]
}

const NOT_FILE_A: u64 = 0xfefefefefefefefe;
const NOT_FILE_B: u64 = 0xfcfcfcfcfcfcfcfc;
const NOT_FILE_G: u64 = 0x3f3f3f3f3f3f3f3f;
const NOT_FILE_H: u64 = 0x7f7f7f7f7f7f7f7f;

pub fn find_knight_attacks(bb: u64) -> u64 {
    let l1 = (bb & NOT_FILE_A) >> 1;
    let l2 = (bb & NOT_FILE_B) >> 2;
    let r1 = (bb & NOT_FILE_H) << 1;
    let r2 = (bb & NOT_FILE_G) << 2;

    (l1 << 16)
        | (r1 << 16)
        | (l1 >> 16)
        | (r1 >> 16)
        | (l2 << 8)
        | (r2 << 8)
        | (l2 >> 8)
        | (r2 >> 8)
}

fn generate_knight(square: u8, board: &Board, state: &State) -> u64 {
    let k = 1u64 << square;

    !board.colored()[state.turn() as usize] & find_knight_attacks(k)
}

pub fn find_pawn_attacks(bb: u64, color: bool) -> u64 {
    match color {
        false => ((bb & NOT_FILE_A) << 7) | ((bb & NOT_FILE_H) << 9),
        true => ((bb & NOT_FILE_A) >> 9) | ((bb & NOT_FILE_H) >> 7),
    }
}

// Promotions not handled here, only when splitting move bb into individual moves
fn generate_pawn(square: u8, board: &Board, state: &State) -> u64 {
    let pawn = 1u64 << square;
    let empty = !board.all();
    let enemy = board.colored()[!state.turn() as usize];

    let mut moves: u64 = 0;

    if !state.turn() {
        // single push
        let single = (pawn << 8) & empty;
        moves |= single;

        // double push
        if (pawn & RANK_2) != 0 && single != 0 {
            moves |= (single << 8) & empty;
        }

        // captures
        moves |= ((pawn & !FILE_A) << 7) & enemy;
        moves |= ((pawn & !FILE_H) << 9) & enemy;

        moves
    } else {
        // single push
        let single = (pawn >> 8) & empty;
        moves |= single;

        // double push
        if (pawn & RANK_7) != 0 && single != 0 {
            moves |= (single >> 8) & empty;
        }

        // captures
        moves |= ((pawn & !FILE_A) >> 9) & enemy;
        moves |= ((pawn & !FILE_H) >> 7) & enemy;

        moves
    }
}

// Correct castle flag bits
const WHITE_KINGSIDE: u8 = 0b1000;
const WHITE_QUEENSIDE: u8 = 0b0100;
const BLACK_KINGSIDE: u8 = 0b0010;
const BLACK_QUEENSIDE: u8 = 0b0001;

// Squares that must be EMPTY for castling (between king and rook)
const WHITE_KINGSIDE_EMPTY: u64 = (1 << 5) | (1 << 6); // f1, g1
const WHITE_QUEENSIDE_EMPTY: u64 = (1 << 1) | (1 << 2) | (1 << 3); // b1, c1, d1
const BLACK_KINGSIDE_EMPTY: u64 = (1 << 61) | (1 << 62); // f8, g8
const BLACK_QUEENSIDE_EMPTY: u64 = (1 << 57) | (1 << 58) | (1 << 59); // b8, c8, d8

// Squares that must NOT be attacked for castling
// (king's path: the square it sits on, passes through, and lands on)
const WHITE_KINGSIDE_SAFE: u64 = (1 << 4) | (1 << 5) | (1 << 6); // e1, f1, g1
const WHITE_QUEENSIDE_SAFE: u64 = (1 << 4) | (1 << 3) | (1 << 2); // e1, d1, c1
const BLACK_KINGSIDE_SAFE: u64 = (1 << 60) | (1 << 61) | (1 << 62); // e8, f8, g8
const BLACK_QUEENSIDE_SAFE: u64 = (1 << 60) | (1 << 59) | (1 << 58); // e8, d8, c8

// Destination squares for castling moves
const WHITE_KINGSIDE_DEST: u64 = 1 << 6; // g1
const WHITE_QUEENSIDE_DEST: u64 = 1 << 2; // c1
const BLACK_KINGSIDE_DEST: u64 = 1 << 62; // g8
const BLACK_QUEENSIDE_DEST: u64 = 1 << 58; // c8

fn generate_castles(board: &Board, state: &State, moveinfo: &MoveInfo) -> u64 {
    struct CastleConfig {
        right: u8,
        empty: u64,
        safe: u64,
        dest: u64,
    }

    const CONFIGS: [[CastleConfig; 2]; 2] = [
        // White
        [
            CastleConfig {
                right: WHITE_KINGSIDE,
                empty: WHITE_KINGSIDE_EMPTY,
                safe: WHITE_KINGSIDE_SAFE,
                dest: WHITE_KINGSIDE_DEST,
            },
            CastleConfig {
                right: WHITE_QUEENSIDE,
                empty: WHITE_QUEENSIDE_EMPTY,
                safe: WHITE_QUEENSIDE_SAFE,
                dest: WHITE_QUEENSIDE_DEST,
            },
        ],
        // Black
        [
            CastleConfig {
                right: BLACK_KINGSIDE,
                empty: BLACK_KINGSIDE_EMPTY,
                safe: BLACK_KINGSIDE_SAFE,
                dest: BLACK_KINGSIDE_DEST,
            },
            CastleConfig {
                right: BLACK_QUEENSIDE,
                empty: BLACK_QUEENSIDE_EMPTY,
                safe: BLACK_QUEENSIDE_SAFE,
                dest: BLACK_QUEENSIDE_DEST,
            },
        ],
    ];

    CONFIGS[state.turn() as usize]
        .iter()
        .fold(0u64, |destinations, cfg| {
            if state.castling() & cfg.right != 0
                && (board.all() & cfg.empty) == 0
                && (moveinfo.attacked_squares & cfg.safe) == 0
            {
                destinations | cfg.dest
            } else {
                destinations
            }
        })
}

// Has to handle annoying special cases (capture out of check, ep revealing self-check), might as well have own logic anyways
// SHOULD do filtering for check cases despite generally other piecegens don't (also its pretty simple here)
// Does not handle normal pins
fn generate_ep(square: u8, board: &Board, state: &State, moveinfo: &MoveInfo) -> u64 {
    let capture_square: u8 = (if state.turn() { 3u8 } else { 4u8 }) * 8 + file(state.epfile());
    let mut final_square = capture_square;

    if state.turn() {
        final_square -= 8;
    } else {
        final_square += 8;
    }

    if 1u64 << final_square & find_pawn_attacks(1u64 << square, state.turn()) == 0 {
        return 0;
    }

    if moveinfo.check
        && (capture_square != moveinfo.check_square)
        && ((1u64 << final_square) & moveinfo.stop_check_squares) == 0
    {
        return 0;
    }

    let king_sq: u8 = board
        .bitboard(PieceType::KING, state.turn())
        .trailing_zeros() as u8;
    if rank(square) == rank(king_sq) {
        let occ: u64 = board.all() ^ (1 << square) ^ (1 << capture_square);
        let check_square: u64 =
            generate_sliding_occupancy(king_sq, occ, false) & (0xFFu64 << (rank(king_sq) * 8));
        let attackers = (board.piece()[PieceType::ROOK as usize]
            | board.piece()[PieceType::QUEEN as usize])
            & board.colored()[!state.turn() as usize];
        if check_square & attackers != 0 {
            return 0u64;
        }
    }

    1u64 << final_square
}
