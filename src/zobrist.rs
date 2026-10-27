use rand::Rng;
use rand::rand_core::SeedableRng;
use rand_pcg::Pcg64;
use std::sync::LazyLock;
use crate::game::state::*;
use crate::game::board::*;

// Constant seed
const SEED: u64 = 676767676767;

static ZOBRIST_BITSTR: LazyLock<[u64; 793]> = LazyLock::new(|| {
    zobrist_init()
});

// Add castling bits to this for key
pub const CASTLING_OFFSET: usize = 768; 
// Add EP bits to this for key
pub const EP_OFFSET: usize = 784;
// Black to move
pub const BLACK_TO_MOVE_OFFSET: usize = 792;

fn zobrist_init() -> [u64; 793] {
    let mut rng = Pcg64::seed_from_u64(SEED); 

    let mut bitstrs: [u64; 793] = [0; 793]; 
    for i in &mut bitstrs {
        *i = rng.next_u64();
    }

    bitstrs
}

// Slow function, only used from starting position / board init
pub fn zobrist_from_board(board: &Board, state: &State) -> u64
{
    let mut key: u64 = 0;

    let mut occupied = board.all();
    while occupied != 0 {
        let square = occupied.trailing_zeros() as usize;
        key ^= ZOBRIST_BITSTR[board.zobrist_idx(square)];
        occupied &= occupied - 1;
    }

    key ^= ZOBRIST_BITSTR[CASTLING_OFFSET + state.castling() as usize];
     
    if state.epfile() != 0b1000 {
        key ^= ZOBRIST_BITSTR[EP_OFFSET + state.epfile() as usize];
    } 

    if state.turn() {
        key ^= ZOBRIST_BITSTR[BLACK_TO_MOVE_OFFSET];
    }  

    key
}

#[inline]
pub fn zobrist_update(key: u64, idx: usize) -> u64 {
    key ^ ZOBRIST_BITSTR[idx]
}

#[cfg(test)]
#[path = "../test/zobrist.rs"]
mod tests;
