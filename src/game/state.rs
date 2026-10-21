use crate::game::datatypes::*;
use crate::{dprintln, utils::*};

// Game state info (depends on multiple moves)
#[derive(Clone)]
pub struct State {
    // Rules (past information required to determine moves)
    castling: u8,
    fiftymove: u16,
    epfile: u8,
    capture_type: u8,
    halfmoves: u32,

    // Stack (will need to store info for undoing a move)
    // Bits 0-3 are castle state (KQkq)
    // Bits 4-6 are EP file bits (if EP enabled)
    // Bits 7-9 are capture bits, if there was a capture, the piece type is in 7-9
    // Index into PieceType (EMPTY being no capture)
    // With 111 being EP possible because you cannot capture + enable EP
    // 10-16 is fiftymove (7 bits, max 100, fits in u32)
    // Promote is stored in move, since we need to know what were going to promote to anyways
    stack: Vec<u32>,
}

impl State {
    pub fn new(castling: u8, fiftymove: u16, epfile: u8, capture_type: u8, halfmoves: u32) -> State {
        State {
            castling,
            fiftymove,
            epfile,
            capture_type: capture_type,
            halfmoves,
            stack: Vec::new(),
        }
    }

    pub fn castling(&self) -> u8 {
        self.castling
    }

    pub fn fiftymove(&self) -> u16 {
        self.fiftymove 
    }

    pub fn epfile(&self) -> u8 {
        self.epfile 
    }

    pub fn capture_type(&self) -> u8 {
        self.capture_type
    }

    pub fn halfmoves(&self) -> u32 {
        self.halfmoves
    }
    
    pub fn turn(&self) -> bool {
        self.halfmoves() % 2 != 0
    }

    pub fn past_epfile(&self) -> u8 {
        ((self.stack.last().copied().unwrap_or(0) >> 4) & 0b111) as u8
    }

    pub fn past_capture_type(&self) -> u8 {
        ((self.stack.last().copied().unwrap_or(0) >> 7) & 0b111) as u8
    }
    
    pub fn update(&mut self, piece_type: PieceType, capture_type: PieceType, cmove: &Move) {
        let turn: bool = (self.halfmoves % 2) != 0;

        // Castling
        // Rights are lost only when a rook leaves, or is captured on, its home corner
        let home_rank: u8 = if turn { 7 } else { 0 };
        if piece_type == PieceType::ROOK && rank(cmove.start) == home_rank && file(cmove.start) % 7 == 0 {
            self.castling &= !(1 << (2 * (!turn as u8) + file(cmove.start) / 7));
        } else if piece_type == PieceType::KING {
            self.castling &= 0b11 << (turn as u8 * 2);
        }

        if capture_type == PieceType::ROOK && rank(cmove.end) == 7 - home_rank && file(cmove.end) % 7 == 0 {
            self.castling &= !(1 << (2 * (turn as u8) + file(cmove.end) / 7));
        }

        // 50 move rule 
        if piece_type == PieceType::PAWN || capture_type != PieceType::EMPTY {
            self.fiftymove = 0;
        } else {
            self.fiftymove += 1;
        }

        // Capture type
        self.capture_type = capture_type as u8;

        // EP file
        if piece_type == PieceType::PAWN && (rank(cmove.start).abs_diff(rank(cmove.end))) == 2 {
            self.epfile = file(cmove.start);
            self.capture_type = PieceType::EP as u8;
        }

        self.halfmoves += 1;
    }

    pub fn push_history(&mut self) {
        let mut val: u32 = (self.castling & 0b1111) as u32;
        val |= ((self.epfile & 0b111) as u32) << 4;
        val |= ((self.capture_type & 0b111) as u32) << 7;
        val |= (self.fiftymove as u32) << 10;
        self.stack.push(val);

    }

    pub fn pop_history(&mut self) {
        let val = self.stack.pop().expect("expect push history before pop history");
        self.castling = (val & 0b1111) as u8;
        self.epfile = ((val >> 4) & 0b111) as u8;
        self.capture_type = ((val >> 7) & 0b111) as u8;
        self.fiftymove = (val >> 10) as u16;

        self.halfmoves -= 1;
    }
}

