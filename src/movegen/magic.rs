use crate::{dprintln, utils::*}; 
use crate::game::datatypes::*;

use serde::{Serialize, Deserialize};
use std::fs;
use std::sync::LazyLock;

pub static MAGICATTACKS: LazyLock<([Magic; 128], Vec<u64>)> = LazyLock::new(|| {
    // A GUI may launch the engine from any working directory.
    let loaded: MagicData = postcard::from_bytes(include_bytes!("../../resources/magic.bin"))
        .expect("invalid embedded magic tables");
    (loaded.magic.try_into().expect("expected 128 magic tables"), loaded.attacks)
});

#[derive(Clone, Copy, Default, PartialEq, Debug, Serialize, Deserialize)]
pub struct Magic {
    pub mask: u64,
    pub magic: u64,
    pub shift: u8,
    pub offset: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct MagicData {
    magic: Vec<Magic>, 
    attacks: Vec<u64>,
}

fn calc_rook_attacks(square: u8, occupied: u64) -> u64 {
    positive_ray(square, occupied, Dir::NORTH) 
        | positive_ray(square, occupied, Dir::EAST) 
        | negative_ray(square, occupied, Dir::SOUTH) 
        | negative_ray(square, occupied, Dir::WEST)
}

fn calc_bishop_attacks(square: u8, occupied: u64) -> u64 {
    positive_ray(square, occupied, Dir::NORTHWEST) 
        | positive_ray(square, occupied, Dir::NORTHEAST) 
        | negative_ray(square, occupied, Dir::SOUTHWEST) 
        | negative_ray(square, occupied, Dir::SOUTHEAST)
}

fn positive_ray(square: u8, occupied: u64, dir: Dir) -> u64{
    let attacks = find_ray(square, dir);
    let blocker = attacks & occupied;
    let block_square = (blocker | 0x8000000000000000).trailing_zeros() as u8;
    attacks ^ find_ray(block_square, dir)
}

fn negative_ray(square: u8, occupied: u64, dir: Dir) -> u64{
    let attacks = find_ray(square, dir);
    let blocker = attacks & occupied;
    let block_square = (blocker | 1).ilog2() as u8;
    attacks ^ find_ray(block_square, dir)
}

fn generate_candidate() -> u64
{
    rand::random::<u64>() & rand::random::<u64>() & rand::random::<u64>()
}

fn test_magic(magic: u64, relevant_bits: u64, relevant_count: u32, occupancys: &Vec<u64>, attacks: &Vec<u64>) -> Option<Vec<u64>> {
    if ((relevant_bits.wrapping_mul(magic)) & 0xFF00000000000000).count_ones() < 6 {
        return None;
    }

    let shift = 64 - relevant_count;
    
    let mut magic_attacks: Vec<u64> = vec![0u64; (2 as usize).pow(relevant_count)];
    let mut greatest_value = 0;

    for i in 0..(2 as usize).pow(relevant_count) {
        let index: usize = ((occupancys[i].wrapping_mul(magic)) >> shift) as usize;

        if greatest_value < index { greatest_value = index; }

        if magic_attacks[index] == 0 {
            magic_attacks[index] = attacks[i];
        } else if magic_attacks[index] == attacks[i] {
            continue;
        } else {
            return None;
        }
    }

    magic_attacks.truncate(greatest_value as usize + 1);
    Some(magic_attacks)
}

pub fn find_magic(square: u8, diag: bool) -> (Magic, Vec<u64>) {
    let relevant_bits = find_relevant(square, diag, true);
    let relevant_count = relevant_bits.count_ones();

    let mut occupancys: Vec<u64> = Vec::with_capacity((2 as usize).pow(relevant_count));
    let mut attacks: Vec<u64> = Vec::with_capacity((2 as usize).pow(relevant_count));

    let mut subset = relevant_bits;

    loop {
        occupancys.push(subset);
        attacks.push(if diag {calc_bishop_attacks(square, subset)} else {calc_rook_attacks(square, subset)});

        subset = subset.wrapping_sub(1) & relevant_bits;

        if subset == relevant_bits { break; }
    }

    loop {
        let magic = generate_candidate();
        if let Some(attacks) = test_magic(magic, relevant_bits, relevant_count, &occupancys, &attacks) {
            return (Magic { mask: relevant_bits, magic: magic, shift: (64 - relevant_count) as u8, offset: 0 }, attacks);
        }
    }

}

pub fn find_all_magic() -> ([Magic; 128], [Vec<u64>; 128]) { 
    let mut magics: [Magic; 128] = [Magic::default(); 128];
    let mut attacks: [Vec<u64>; 128] = std::array::from_fn(|_| Vec::new());

    for diag in 0..2 {
        for i in 0..64 {
            let (mag, att) = find_magic(i, diag != 0);
            dprintln!("Magic: {}, Size: {}, Square: {}", mag.magic, att.len() * 8, Square {square: i});
            
            magics[(diag * 64 + i) as usize] = mag;
            attacks[(diag * 64 + i) as usize] = att;
        }
    }

    (magics, attacks)
}

pub fn flatten_attacks(magics: [Magic; 128], attacks: [Vec<u64>; 128]) -> ([Magic; 128], Vec<u64>) {
    let mut running_off: usize = 0;
    let mut flat_magics: [Magic; 128] = [Magic::default(); 128];
    let mut flat_attacks: Vec<u64> = Vec::new();

    for i in 0..128 {
        flat_magics[i] = magics[i];
        flat_magics[i].offset = running_off;
        running_off += attacks[i].len();
        flat_attacks.extend(&attacks[i]);
    }
    
    (flat_magics, flat_attacks)
}

pub fn save_magic(path: &str, magics: [Magic; 128], attacks: [Vec<u64>; 128]) -> Result<(), Box<dyn std::error::Error>> {   
    let (flat_magics, flat_attacks) = flatten_attacks(magics, attacks);

    let save = MagicData { magic: flat_magics.to_vec(), attacks: flat_attacks };

    let bytes = postcard::to_stdvec(&save)?;
    fs::write(path, bytes)?;
    Ok(())
}

pub fn load_magic(path: &str) -> Result<([Magic; 128], Vec<u64>), Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;

    let loaded: MagicData = postcard::from_bytes(&bytes)?;
    let magic: [Magic; 128] = loaded.magic.try_into().unwrap();
    Ok((magic, loaded.attacks))
}

