use super::*;

#[test]
fn packed_entries_preserve_scores_moves_and_bounds() {
    for nodetype in [
        NodeType::NONE,
        NodeType::EXACT,
        NodeType::LOWER,
        NodeType::UPPER,
    ] {
        for score in [i32::MIN, -1_000_000, 0, 1_000_000, i32::MAX] {
            for piece in [
                PieceType::EMPTY,
                PieceType::KNIGHT,
                PieceType::BISHOP,
                PieceType::ROOK,
                PieceType::QUEEN,
            ] {
                let entry = TranspositionEntry {
                    zobrist: 0x1234_5678_9ABC_DEF0,
                    score,
                    depth: 255,
                    cmove: Move {
                        start: 48,
                        end: 57,
                        piece,
                    },
                    nodetype,
                };
                assert_eq!(transposition_unpack(transposition_pack(entry)), entry);
            }
        }
    }
}

#[test]
fn table_retrieves_stored_entries_and_rejects_collisions() {
    let mut table = TranspositionTable::new();
    let entry = TranspositionEntry {
        zobrist: 42,
        score: -350,
        depth: 3,
        cmove: Move {
            start: 12,
            end: 28,
            piece: PieceType::EMPTY,
        },
        nodetype: NodeType::EXACT,
    };
    assert_eq!(table.get_check_entry(entry.zobrist), None);
    table.replace_entry(entry);
    assert_eq!(table.get_check_entry(entry.zobrist), Some(entry));
    assert_eq!(
        table.get_check_entry(entry.zobrist + TRANSPOSITION_TABLE_ENTRIES),
        None
    );
}
