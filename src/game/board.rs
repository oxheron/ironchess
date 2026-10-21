use crate::game::datatypes::*;
use crate::game::state::*;
use crate::utils::*;

use std::collections::HashMap;
use std::convert::TryFrom;

use crate::dprintln;

pub struct Board {
    // Bitboards
    all: u64,
    colored: [u64; 2],
    piece: [u64; 6],

    // Pieces index by square
    // Only valid on squares that contain a piece - bitboards are source of truth
    pieces: [Piece; 64],
}

impl Board {
    pub fn new(all: u64, colored: [u64; 2], piece: [u64; 6], pieces: [Piece; 64]) -> Board {
        Board { all, colored, piece, pieces }
    }

    pub fn all(&self) -> u64 {
        self.all
    }

    pub fn colored(&self) -> &[u64; 2] {
        &self.colored
    }

    pub fn piece(&self) -> &[u64; 6] {
        &self.piece
    }

    pub fn pieces(&self) -> &[Piece; 64] {
        &self.pieces
    }

    pub fn bitboard(&self, pt: PieceType, side: bool) -> u64 {
        &self.piece[pt as usize] & &self.colored[side as usize]
    }

    pub fn diagnose(&self) {
        println!("All:");
        print_bitboard(self.all);
        println!("White:");
        print_bitboard(self.colored[0]);
        println!("Black:");
        print_bitboard(self.colored[1]);

        for piece in self.piece {
            print_bitboard(piece);
        } 
    }


    // Updates only the board - does not care about legality
    // Properly updates state and board (assuming the move is legal)
    pub fn do_move(&mut self, mut state: State, cmove: &Move) -> State {
        // setup
        let turn: bool = (state.halfmoves() % 2) != 0;
        let piece: Piece = self.pieces[cmove.start as usize];
        let ep: bool = piece.piece_type == PieceType::PAWN 
            && file(cmove.start) != file(cmove.end) 
            && state.epfile() == file(cmove.end) 
            && rank(cmove.end) == if turn {2} else {5}
            && state.capture_type() == PieceType::EP as u8;
        let capture_square: u8 = if ep {rank(cmove.start) * 8 + file(cmove.end)} else {cmove.end};
        let mut capture_type: PieceType = PieceType::EMPTY;
        let result_piece: PieceType = if cmove.piece != PieceType::EMPTY {cmove.piece} else {piece.piece_type};

        // println!("capture_square {}", capture_square);
        // if piece.piece_type == PieceType::KING && cmove.end == 60 { panic!("illegal"); } 
        // print_bitboard(self.all());

        // bitboard
        let start: u64 = 1 << cmove.start;
        let end: u64 = 1 << cmove.end;

        // capture (BB + hashmap)
        if self.colored[!turn as usize] & 1u64 << capture_square > 0 {
            // print
            let capture_square_bb: u64 = 1 << capture_square;
            self.colored[!turn as usize] &= !capture_square_bb;
            capture_type = self.pieces[capture_square as usize].piece_type;
            self.piece[capture_type as usize] &= !capture_square_bb;
        }

        self.colored[turn as usize] = (self.colored[turn as usize] | end) & !start;
        self.piece[piece.piece_type as usize] &= !start;
        self.piece[result_piece as usize] |= end;


        // hashmap
        self.pieces[cmove.end as usize] = Piece {piece_type: result_piece, square: cmove.end, color: Color::from_bool(turn)};

        // castling
        if piece.piece_type == PieceType::KING && file(cmove.end).abs_diff(file(cmove.start)) > 1
        {
            let kingside = file(cmove.end) > file(cmove.start);
            let ssquare = rank(cmove.start) * 8 + if kingside {7} else {0};
            let esquare = rank(cmove.start) * 8 + if kingside {5} else {3};

            let start: u64 = 1 << ssquare;
            let end: u64 = 1 << esquare;

            self.colored[turn as usize] = (self.colored[turn as usize] | end) & !start;
            self.piece[PieceType::ROOK as usize] = (self.piece[PieceType::ROOK as usize] | end) & !start;

            self.pieces[esquare as usize] = Piece {piece_type: PieceType::ROOK, square: esquare, color: Color::from_bool(turn)};
            self.pieces[ssquare as usize] = Piece::default();
        }

        // update all pieces
        self.all = self.colored[0] | self.colored[1];

        // Board state
        state.push_history();

        // new state (besides capture type)
        state.update(piece.piece_type, capture_type, cmove);

        state
    }

    pub fn do_unmove(&mut self, mut state: State, cmove: &Move) -> State {
        // setup
        let turn: bool = ((state.halfmoves() - 1) % 2) != 0;
        let piece: Piece = self.pieces[cmove.end as usize];
        let capture_type: PieceType = PieceType::try_from(state.capture_type()).expect("corrupted capture type or stack");
        let start_piece: PieceType = if cmove.piece != PieceType::EMPTY {PieceType::PAWN} else {piece.piece_type};
        let ep: bool = state.past_capture_type() == 0b111 
            && state.past_epfile() == file(cmove.end) 
            && rank(cmove.end) == (if turn {2} else {5})
            && capture_type == PieceType::PAWN
            && start_piece == PieceType::PAWN;
        let capture_square: u8 = if ep {rank(cmove.start) * 8 + file(cmove.end)} else {cmove.end};

        // bitboard
        let start: u64 = 1 << cmove.start;
        let end: u64 = 1 << cmove.end;

        self.colored[turn as usize] = (self.colored[turn as usize] | start) & !end;
        self.piece[piece.piece_type as usize] &= !end;
        self.piece[start_piece as usize] |= start;

        // hashmap
        self.pieces[cmove.start as usize] = Piece {piece_type: start_piece, square: cmove.start, color: Color::from_bool(turn)};
        
        // capture (0b111 used to indicate readiness of EP)
        // dprintln!("{:?}", capture_type);
        if capture_type != PieceType::EMPTY && capture_type != PieceType::EP {
            let capture_square_bb: u64 = 1 << capture_square;
            self.colored[!turn as usize] |= capture_square_bb;
            self.piece[capture_type as usize] |= capture_square_bb;
            self.pieces[capture_square as usize] = Piece {piece_type: capture_type, square: capture_square, color: Color::from_bool(!turn)};
        } 

        // castling
        if piece.piece_type == PieceType::KING && file(cmove.end).abs_diff(file(cmove.start)) > 1
        {
            let kingside = file(cmove.end) > file(cmove.start);
            let ssquare = rank(cmove.start) * 8 + if kingside {7} else {0};
            let esquare = rank(cmove.start) * 8 + if kingside {5} else {3};

            let start: u64 = 1 << ssquare;
            let end: u64 = 1 << esquare;

            self.colored[turn as usize] = (self.colored[turn as usize] & !end) | start;
            self.piece[PieceType::ROOK as usize] = (self.piece[PieceType::ROOK as usize] & !end) | start;

            self.pieces[ssquare as usize] = Piece {piece_type: PieceType::ROOK, square: ssquare, color: Color::from_bool(turn)};
            self.pieces[esquare as usize] = Piece::default();
        }

        // dprintln!("{:?}", start_piece);
        // print_bitboard(self.bitboard(PieceType::KING, false));

        // update all pieces
        self.all = self.colored[0] | self.colored[1];

        state.pop_history();

        state
    }
}

