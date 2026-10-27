use num_enum::TryFromPrimitive;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Debug, TryFromPrimitive, Default, Hash, Eq)]
#[repr(u8)]
pub(crate) enum PieceType {
    PAWN,
    KNIGHT,
    BISHOP,
    ROOK,
    QUEEN,
    KING,
    #[default] EMPTY,
    EP,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub(crate) enum Color {
    #[default] WHITE,
    BLACK,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Dir {
    NORTH,
    NORTHEAST,
    EAST,
    SOUTHEAST,
    SOUTH,
    SOUTHWEST,
    WEST,
    NORTHWEST,
}

#[derive(Clone, Copy, PartialEq, Debug, TryFromPrimitive, Default, Hash, Eq)]
#[repr(u8)]
pub(crate) enum NodeType {
    #[default] NONE,
    EXACT,
    LOWER,
    UPPER
}

pub(crate) const ROOK_DIRS: [Dir; 4] = [
    Dir::NORTH, 
    Dir::EAST, 
    Dir::SOUTH, 
    Dir::WEST,
];

pub(crate) const BISHOP_DIRS: [Dir; 4] = [
    Dir::NORTHEAST, 
    Dir::SOUTHEAST,
    Dir::SOUTHWEST,
    Dir::NORTHWEST,
];

impl Color {
    pub fn from_bool(color: bool) -> Color {
        if color {Color::BLACK} else {Color::WHITE}
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub(crate) struct Piece {
    pub piece_type: PieceType,
    pub square: u8,
    pub color: Color,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub(crate) struct Move {
    pub start: u8,
    pub end: u8,
    pub piece: PieceType,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sq = |s: u8| format!("{}{}", (b'a' + (s % 8)) as char, (b'1' + (s / 8)) as char);
        let promo = match self.piece {
            PieceType::QUEEN  => "q",
            PieceType::ROOK   => "r",
            PieceType::BISHOP => "b",
            PieceType::KNIGHT => "n",
            _                 => "",
        };
        write!(f, "{}{}{}", sq(self.start), sq(self.end), promo)
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
// Info not required to know board state, but useful for movegen
pub struct MoveInfo {
    pub pins: u64, 
    pub pin_rays: u64, 

    // 0, 1, or >1 (if greater than one may not be accurate)
    pub check_counts: u8,
    pub check: bool,
    pub double_check: bool,
    pub check_square: u8, 
    pub stop_check_squares: u64,
    

    pub attacked_squares: u64,
}

