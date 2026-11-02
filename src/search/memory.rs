use crate::movegen::MoveBitset;
use crate::game::datatypes::*;

#[derive(PartialEq, Default)]
pub struct SearchFrame {
    pub move_bitset: MoveBitset,
    pub ordered_moves: Vec<Move>,  
}
