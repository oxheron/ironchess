use core::panic;

use crate::{game::{board::*, datatypes::*, state::*}, movegen::{generate_sliding_occupancy, find_knight_attacks, find_pawn_attacks, find_king_attacks}};
use crate::utils::*;

// Uses bitboard and finds attacks, not necessarily legal moves

pub fn generate_info(board: &Board, state: &State) -> MoveInfo {
    let mut moveinfo: MoveInfo = MoveInfo::default(); 
    
    // println!("{}", state.turn());
    // print_bitboard(board.piece()[PieceType::KING as usize]);
    let king_bb: u64 = board.bitboard(PieceType::KING, state.turn());
    // print_bitboard(king_bb);
    let king_sq: u8 = king_bb.trailing_zeros() as u8; let king_knight_attacks: u64 = find_knight_attacks(king_bb);
    let king_pawn_attacks: u64 = find_pawn_attacks(king_bb, state.turn());

    moveinfo = checks_and_attacks(king_bb, board, state, moveinfo, PieceType::BISHOP);
    moveinfo = checks_and_attacks(king_bb, board, state, moveinfo, PieceType::ROOK);
    moveinfo = checks_and_attacks(king_bb, board, state, moveinfo, PieceType::QUEEN);

    // Add pawn + knight + king 
    moveinfo.attacked_squares |= find_knight_attacks(board.bitboard(PieceType::KNIGHT, !state.turn()))
        | find_pawn_attacks(board.bitboard(PieceType::PAWN, !state.turn()), !state.turn())
        | find_king_attacks(board.bitboard(PieceType::KING, !state.turn()));

    // Check pawn and knight for checks
    let attacking_pawns: u64 = board.bitboard(PieceType::PAWN, !state.turn()) & king_pawn_attacks;
    moveinfo.check_counts += attacking_pawns.count_ones() as u8;
    if attacking_pawns != 0 {
        moveinfo.check_square = attacking_pawns.trailing_zeros() as u8;
    }

    let attacking_knights: u64 = board.bitboard(PieceType::KNIGHT, !state.turn()) & king_knight_attacks;
    moveinfo.check_counts += attacking_knights.count_ones() as u8;
    if attacking_knights != 0 {
        moveinfo.check_square = attacking_knights.trailing_zeros() as u8;
    }

    moveinfo.check = moveinfo.check_counts > 0;
    moveinfo.double_check = moveinfo.check_counts > 1;

    let pininfo = calculate_pins(king_sq, board, state);
    moveinfo.pins = pininfo.0;
    moveinfo.pin_rays = pininfo.1;

    moveinfo
}

fn checks_and_attacks(king_bb: u64, board: &Board, state: &State, info: MoveInfo, pt: PieceType) -> MoveInfo {
    let mut piece_iter = board.bitboard(pt, !state.turn());
    let mut info = info; 
     
    while piece_iter != 0 {
        let sq = piece_iter.trailing_zeros() as u8;
        let attacks = match pt {
            PieceType::BISHOP => generate_sliding_occupancy(sq, board.all() & !king_bb, true),
            PieceType::ROOK => generate_sliding_occupancy(sq, board.all() & !king_bb, false),
            PieceType::QUEEN => generate_sliding_occupancy(sq, board.all() & !king_bb, false) | generate_sliding_occupancy(sq, board.all() & !king_bb, true), 
            _ => panic!("Not a slider"),
        };

        info.attacked_squares |= attacks; 

        if king_bb & attacks > 0 {
            info.check_counts += 1;
            info.check_square = sq;
            let king_sq = king_bb.trailing_zeros() as u8;
            info.stop_check_squares |= BETWEEN[64 * king_sq as usize + sq as usize];
        }

        piece_iter &= piece_iter - 1; 
    }

    info
}

fn calculate_pins(square: u8, board: &Board, state: &State) -> (u64, u64) {
    let king_sliding_attacks: u64 = generate_sliding_occupancy(square, board.all(), false) | generate_sliding_occupancy(square, board.all(), true);
    let occ_xray: u64 = board.all() ^ (king_sliding_attacks & board.colored()[state.turn() as usize]); 
    let king_xray: u64 = generate_sliding_occupancy(square, occ_xray, false) | generate_sliding_occupancy(square, occ_xray, true);
    let xray_only: u64 = king_xray ^ king_sliding_attacks;
    let enemy_sliders: u64 = (board.piece()[PieceType::ROOK as usize] 
        | board.piece()[PieceType::QUEEN as usize] 
        | board.piece()[PieceType::BISHOP as usize])
        & board.colored()[!state.turn() as usize];

    let pinners: u64 = xray_only & enemy_sliders; // enemy rooks/queens or bishops/queens

    let mut pins: u64 = 0; 
    let mut pin_rays: u64 = 0;

    let mut pinner_iter = pinners;
    while pinner_iter != 0 {
        let pinner_sq = pinner_iter.trailing_zeros() as u8;

        if board.pieces()[pinner_sq as usize].piece_type == PieceType::ROOK && diag(pinner_sq, square) {
            pinner_iter &= pinner_iter - 1; 
            continue;
        }

        if board.pieces()[pinner_sq as usize].piece_type == PieceType::BISHOP && !diag(pinner_sq, square) {
            pinner_iter &= pinner_iter - 1; 
            continue;
        }

        let pin_ray: u64 = BETWEEN[64 * (square as usize) + pinner_sq as usize] | 1u64 << pinner_sq;
    
        // The pinned piece is on the pin ray and is friendly
        pins |= (pin_ray & board.colored()[state.turn() as usize]);
        pin_rays |= pin_ray;

        pinner_iter &= pinner_iter - 1; 
    }

    (pins, pin_rays)
}

