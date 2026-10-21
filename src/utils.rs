use crate::game::datatypes::*;

use std::sync::LazyLock;
use std::fmt;

// Only used for format printing
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct Square { 
    pub square: u8,
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", FILE_AS_CHAR[file(self.square) as usize], 1 + rank(self.square))
    }
}

#[macro_export]
macro_rules! dprintln {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!($($arg)*);
    };
}

pub(crate) fn rank(square: u8) -> u8 {
    square / 8 
}

pub(crate) fn file(square: u8) -> u8 {
    square % 8
}

pub const FILE_AS_CHAR: [char; 8] = [
    'a',
    'b',
    'c',
    'd',
    'e',
    'f',
    'g',
    'h',
];

pub const FILE_A: u64 = 0x0101010101010101;
pub const FILE_H: u64 = 0x8080808080808080;
pub const RANK_1: u64 = 0x00000000000000FF;
pub const RANK_8: u64 = 0xFF00000000000000;
pub const RANK_2: u64 = 0x000000000000FF00;
pub const RANK_7: u64 = 0x00FF000000000000;

const DIRS_TO_OFFSET: [(i8, i8); 8] = [
    (0, 1), 
    (1, 1), 
    (1, 0), 
    (1, -1), 
    (0, -1), 
    (-1, -1), 
    (-1, 0), 
    (-1, 1)
];

fn gen_ray(square: u8, dx: i8, dy: i8) -> u64 {
    let mut attacks = 0u64;

    let mut file = (square % 8) as i8;
    let mut rank = (square / 8) as i8;

    loop {
        file += dx;
        rank += dy;

        if file < 0 || file >= 8 || rank < 0 || rank >= 8 {
            break;
        }

        let sq = (rank * 8 + file) as u64;
        attacks |= 1u64 << sq;
    }

    attacks
}

pub(crate) fn find_ray(square: u8, dir: DIR) -> u64 {
    static RAYS: LazyLock<[Vec<u64>; 8]> =  LazyLock::new(|| {
        let mut tmp: [Vec<u64>; 8] = std::array::from_fn(|_| Vec::new()); 

        for i in 0..8 {
            for j in 0..64 {
                tmp[i].push(gen_ray(j, DIRS_TO_OFFSET[i].0, DIRS_TO_OFFSET[i].1))
            }
        }

        tmp
    });
    

    RAYS[dir as usize][square as usize]
}

fn edges_map(square: u8) -> u64 {
    let mut edges_map: u64 = 0;
    if rank(square) != 0 {
        edges_map |= RANK_1; 
    }

    if rank(square) != 7 {
        edges_map |= RANK_8; 

    }

    if file(square) != 0 {
        edges_map |= FILE_A;
    }

    if file(square) != 7 {
        edges_map |= FILE_H;
    }
    
    edges_map
}

pub(crate) fn find_relevant(square: u8, diag: bool, exclude_edges: bool) -> u64 {
    let dirs = if diag { BISHOP_DIRS } else { ROOK_DIRS };

    let mut attack = 0;

    for dir in dirs {
        attack |= find_ray(square, dir);
    }

    let edges_map = if exclude_edges { edges_map(square) } else { 0 };
    attack & !edges_map
}

fn generate_between_table() -> [u64; 64*64] {
    let mut between = [0u64; 64*64];
    
    for sq1 in 0..64 {
        for sq2 in 0..64 {
            if sq1 == sq2 { continue; }
            
            let r1 = sq1 / 8;
            let f1 = sq1 % 8;
            let r2 = sq2 / 8;
            let f2 = sq2 % 8;
            
            let dr = (r2 as i32 - r1 as i32).signum();
            let df = (f2 as i32 - f1 as i32).signum();
            
            // only valid if on same rank, file, or diagonal
            let on_ray = r1 == r2                        // same rank
                      || f1 == f2                        // same file
                      || (r2 as i32 - r1 as i32).abs() == (f2 as i32 - f1 as i32).abs(); // diagonal
            
            if !on_ray { continue; }
            
            let mut r = r1 as i32 + dr;
            let mut f = f1 as i32 + df;
            
            while (r, f) != (r2 as i32, f2 as i32) {
                between[sq1 * 64 + sq2] |= 1u64 << (r * 8 + f);
                r += dr;
                f += df;
            }
        }
    }
    
    between
}

pub static BETWEEN: LazyLock<[u64; 64*64]> =  LazyLock::new(|| {
    generate_between_table()
});

// Full line through two aligned squares, extended to the board edges in both
// directions. Includes both endpoint squares. Empty if the squares are not on
// a shared rank, file, or diagonal.
fn generate_line_table() -> [u64; 64*64] {
    let mut line = [0u64; 64*64];

    for sq1 in 0..64 {
        for sq2 in 0..64 {
            if sq1 == sq2 { continue; }

            let r1 = sq1 / 8;
            let f1 = sq1 % 8;
            let r2 = sq2 / 8;
            let f2 = sq2 % 8;

            let dr = (r2 as i32 - r1 as i32).signum();
            let df = (f2 as i32 - f1 as i32).signum();

            // only valid if on same rank, file, or diagonal
            let on_ray = r1 == r2                        // same rank
                      || f1 == f2                        // same file
                      || (r2 as i32 - r1 as i32).abs() == (f2 as i32 - f1 as i32).abs(); // diagonal

            if !on_ray { continue; }

            // walk from sq1 to each edge, once forward (covering sq1 itself
            // and sq2) and once backward
            let mut r = r1 as i32;
            let mut f = f1 as i32;

            while (0..8).contains(&r) && (0..8).contains(&f) {
                line[sq1 * 64 + sq2] |= 1u64 << (r * 8 + f);
                r += dr;
                f += df;
            }

            r = r1 as i32 - dr;
            f = f1 as i32 - df;

            while (0..8).contains(&r) && (0..8).contains(&f) {
                line[sq1 * 64 + sq2] |= 1u64 << (r * 8 + f);
                r -= dr;
                f -= df;
            }
        }
    }

    line
}

pub static LINE: LazyLock<[u64; 64*64]> =  LazyLock::new(|| {
    generate_line_table()
});

pub fn diag(sq1: u8, sq2: u8) -> bool {
    let dr = rank(sq2) as i32 - rank(sq1) as i32;
    let df = file(sq2) as i32 - file(sq1) as i32;
    dr.abs() == df.abs()
}

pub fn print_bitboard(bb: u64) {
    for rank in (0..8).rev() {
        for file in 0..8 {
            let square = rank * 8 + file;

            if (bb >> square) & 1 == 1 {
                print!("1 ");
            } else {
                print!(". ");
            }
        }

        println!();
    }
     
    println!();
}
