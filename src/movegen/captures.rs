use crate::game::{board::*, state::*};
use crate::movegen::*;

// MVV-LVA generation ()
// Eventually, switch to SEE for ordering.
pub fn generate_mvv_lva_bitboards(board: &Board, state: &State) -> Vec<(u8, u64)> {
    let mut attackers: [(u8, u64); 16] = [(0, 0); 16];
    let mut attacker_count = 0;

    // Store each attacker's square and attacks in least valuable attacker order.
    for piece_type in [
        PieceType::PAWN,
        PieceType::KNIGHT,
        PieceType::BISHOP,
        PieceType::ROOK,
        PieceType::QUEEN,
        PieceType::KING,
    ] {
        let mut piece_iter = board.bitboard(piece_type, state.turn());
        while piece_iter != 0 {
            let square = piece_iter.trailing_zeros() as u8;
            let piece_bb = 1u64 << square;
            let attacks = match piece_type {
                PieceType::PAWN => find_pawn_attacks(piece_bb, state.turn()),
                PieceType::KNIGHT => find_knight_attacks(piece_bb),
                PieceType::BISHOP => generate_sliding_occupancy(square, board.all(), true),
                PieceType::ROOK => generate_sliding_occupancy(square, board.all(), false),
                PieceType::QUEEN => {
                    generate_sliding_occupancy(square, board.all(), true)
                        | generate_sliding_occupancy(square, board.all(), false)
                }
                PieceType::KING => find_king_attacks(piece_bb),
                _ => unreachable!("only board piece types are iterated"),
            };

            attackers[attacker_count] = (square, attacks);
            attacker_count += 1;
            piece_iter &= piece_iter - 1;
        }
    }

    let mut moves_bb: Vec<(u8, u64)> = Vec::new();

    // Most valuable victim first, then least valuable attacker.
    for victim in [
        PieceType::QUEEN,
        PieceType::ROOK,
        PieceType::BISHOP,
        PieceType::KNIGHT,
        PieceType::PAWN,
    ] {
        let victims = board.bitboard(victim, !state.turn());
        for &(square, attacks) in &attackers[..attacker_count] {
            let captures = victims & attacks;
            if captures != 0 {
                moves_bb.push((square, captures));
            }
        }
    }

    // Append EP to end, for correctness. Just find possible capture squares
    if state.epfile() != 0b1000 {
        let epsquare = state.epfile() + 
            if state.turn() {8 * 2} else {8 * 5};
        
        let mut possible_bb = find_pawn_attacks(1 << epsquare, !state.turn()) 
            & board.bitboard(PieceType::PAWN, state.turn());

        while possible_bb != 0 {
            let square = possible_bb.trailing_zeros() as u8;
            moves_bb.push((square, 1 << epsquare));

            possible_bb &= possible_bb - 1;
        }
    }

    moves_bb
}

#[cfg(test)]
#[path = "../../test/captures.rs"]
mod tests;
