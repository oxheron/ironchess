use crate::game::datatypes::*;

const TRANSPOSITION_TABLE_ENTRIES: u64 = 4000000;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct TranspositionEntry {
    pub zobrist: u64,
    pub score: i32,
    pub depth: u32, 
    pub cmove: Move,
    pub nodetype: NodeType,    
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct PackedTranspositionEntry {
    zobrist: u64,
    score: i32,
    depthmovetype: u32, // depth, move, nodetype. packed
}

pub fn transposition_pack(entry: TranspositionEntry) -> PackedTranspositionEntry {
    let mut packed: PackedTranspositionEntry = PackedTranspositionEntry::default();
    packed.zobrist = entry.zobrist;
    packed.score = entry.score;
    packed.depthmovetype |= (0xFF & entry.depth as u32);
    packed.depthmovetype |= (0xFF & entry.cmove.start as u32) << 8;
    packed.depthmovetype |= (0xFF & entry.cmove.end as u32) << 16;
    packed.depthmovetype |= (0x3F & entry.cmove.piece as u32) << 24;
    packed.depthmovetype |= (0x03 & entry.nodetype as u32) << 30;
    packed
}

pub fn transposition_unpack(packed: PackedTranspositionEntry) -> TranspositionEntry {
    let mut entry = TranspositionEntry::default();
    entry.zobrist = packed.zobrist;
    entry.score = packed.score;
    entry.depth = (0xFF & packed.depthmovetype);

    let mut cmove = Move::default();
    cmove.start = ((0xFF00 & packed.depthmovetype) >> 8) as u8;
    cmove.end = ((0xFF0000 & packed.depthmovetype) >> 16) as u8;
    if let Ok(pt) = PieceType::try_from(((0x3F000000 & packed.depthmovetype) >> 24) as u8) {
        cmove.piece = pt;   
    }
    else {
        println!("Invalid enum conversion");
    }

    entry.cmove = cmove;

    if let Ok(nt) = NodeType::try_from(((0xC0000000 & packed.depthmovetype) >> 30) as u8) {
        entry.nodetype = nt;
    }
    else {
        println!("Invalid enum conversion");
    }

    
    entry
}

#[cfg(test)]
#[path = "../../test/transposition.rs"]
mod tests;

pub struct TranspositionTable {
    table: Box<[PackedTranspositionEntry; TRANSPOSITION_TABLE_ENTRIES as usize]>,
    table_writes: u64, 
}

impl TranspositionTable {
    pub fn new() -> TranspositionTable {
        let tmp: Box<[PackedTranspositionEntry]> = 
            vec![PackedTranspositionEntry::default(); TRANSPOSITION_TABLE_ENTRIES as usize].into_boxed_slice();
    
        TranspositionTable {
            table: tmp.try_into().expect("Size mismatch"), 
            table_writes: 0
        }
    }

    // Gets entry, does not check for index collisions
    pub fn get_entry(&self, zobrist: u64) -> TranspositionEntry {
        transposition_unpack(self.table[(zobrist % TRANSPOSITION_TABLE_ENTRIES) as usize])
    }

    // Checks for index collisions, returns optional
    pub fn get_check_entry(&self, zobrist: u64) -> Option<TranspositionEntry> {
        if self.table[(zobrist % TRANSPOSITION_TABLE_ENTRIES) as usize].depthmovetype != 0 
            && self.table[(zobrist % TRANSPOSITION_TABLE_ENTRIES) as usize].zobrist == zobrist {
            Some(transposition_unpack(self.table[(zobrist % TRANSPOSITION_TABLE_ENTRIES) as usize]))
        } else {
            None
        }
    }

    fn set_entry(&mut self, entry: TranspositionEntry) {
        self.table[(entry.zobrist % TRANSPOSITION_TABLE_ENTRIES) as usize] = transposition_pack(entry);
        #[cfg(debug_assertions)]
        {
            self.table_writes += 1;
        }
    }

    pub fn replace_entry(&mut self, entry: TranspositionEntry) {
        // if empty or this depth greater, then replace
        let cur = self.get_entry(entry.zobrist);
        if cur.nodetype == NodeType::NONE || entry.depth > cur.depth {
            self.set_entry(entry);
        }
    }

    #[cfg(debug_assertions)]
    pub fn table_writes(&self) -> u64 {
        self.table_writes        
    }
}
