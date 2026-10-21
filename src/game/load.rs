use crate::dprintln;
use crate::game::board::Board;
use crate::game::state::State;
use crate::game::datatypes::*;

// Castling bits: K=bit3, Q=bit2, k=bit1, q=bit0
// colored[0]=white, colored[1]=black
// piece[PieceType as usize], epfile=8 means no EP

pub fn load_fen(fen: &str) -> (Board, State) {
    let parts: Vec<&str> = fen.split_whitespace().collect();
    assert!(parts.len() >= 6, "FEN must have 6 fields");

    let board = parse_placement(parts[0]);
    let active_color = parts[1];
    let castling = parse_castling(parts[2]);
    let epfile = parse_ep(parts[3]);
    let mut capture_type = PieceType::EMPTY;
    if epfile != 0b1000 {
        capture_type = PieceType::EP;
    }
    let fiftymove: u16 = parts[4].parse().expect("invalid halfmove clock");
    let fullmove: u32 = parts[5].parse().expect("invalid fullmove number");

    let halfmoves = (fullmove - 1) * 2 + if active_color == "b" { 1 } else { 0 };
    let state = State::new(castling, fiftymove, epfile, capture_type as u8, halfmoves);

    (board, state)
}

fn parse_placement(placement: &str) -> Board {
    let mut pieces: [Piece; 64] = [Piece::default(); 64];
    let mut all: u64 = 0;
    let mut colored: [u64; 2] = [0; 2];
    let mut piece_bb: [u64; 6] = [0; 6];

    // FEN ranks go 8→1 (index 7→0), files a→h (index 0→7)
    let mut rank: u8 = 7;
    let mut file: u8 = 0;

    for ch in placement.chars() {
        match ch {
            '/' => {
                rank -= 1;
                file = 0;
            }
            '1'..='8' => {
                file += ch as u8 - b'0';
            }
            _ => {
                let color = if ch.is_uppercase() { Color::WHITE } else { Color::BLACK };
                let piece_type = match ch.to_ascii_lowercase() {
                    'p' => PieceType::PAWN,
                    'n' => PieceType::KNIGHT,
                    'b' => PieceType::BISHOP,
                    'r' => PieceType::ROOK,
                    'q' => PieceType::QUEEN,
                    'k' => PieceType::KING,
                    c   => panic!("unknown piece char '{c}' in FEN"),
                };

                let square = rank * 8 + file;
                let bb = 1u64 << square;
                let color_idx = (color == Color::BLACK) as usize;

                all |= bb;
                colored[color_idx] |= bb;
                piece_bb[piece_type as usize] |= bb;
                pieces[square as usize] = Piece { piece_type, square, color };

                file += 1;
            }
        }
    }

    Board::new(all, colored, piece_bb, pieces)
}

fn parse_castling(s: &str) -> u8 {
    if s == "-" {
        return 0;
    }
    let mut castling: u8 = 0;
    for ch in s.chars() {
        castling |= match ch {
            'K' => 0b1000,
            'Q' => 0b0100,
            'k' => 0b0010,
            'q' => 0b0001,
            _   => panic!("invalid castling char '{ch}' in FEN"),
        };
    }
    castling
}

// Returns the ep file (0-7), or 8 as sentinel for no EP.
fn parse_ep(s: &str) -> u8 {
    if s == "-" {
        return 8;
    }
    let file_char = s.chars().next().expect("empty EP field in FEN");
    match file_char {
        'a'..='h' => file_char as u8 - b'a',
        _ => panic!("invalid EP file '{file_char}' in FEN"),
    }
}
