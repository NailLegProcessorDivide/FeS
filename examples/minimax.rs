use std::cmp::max;

use fes::{
    bit_board::{BitBoardGame, BitBoardGameMove},
    game::{ChessGame, Move},
};

use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

#[derive(Debug)]
struct SideZobristKeys {
    pub pawn_keys: [u64; 64],
    pub knight_keys: [u64; 64],
    pub bishop_keys: [u64; 64],
    pub rook_keys: [u64; 64],
    pub queen_keys: [u64; 64],
    pub king_keys: [u64; 64],
    pub enpassant_keys: [u64; 8],
    pub kingside_key: u64,
    pub queenside_key: u64,
}

#[derive(Debug)]
struct ZobristKeys {
    pub white_keys: SideZobristKeys,
    pub black_keys: SideZobristKeys,
}

impl SideZobristKeys {
    pub fn new(rng: &mut impl RngCore) -> Self {
        Self {
            pawn_keys: core::array::from_fn(|_| rng.next_u64()),
            knight_keys: core::array::from_fn(|_| rng.next_u64()),
            bishop_keys: core::array::from_fn(|_| rng.next_u64()),
            rook_keys: core::array::from_fn(|_| rng.next_u64()),
            queen_keys: core::array::from_fn(|_| rng.next_u64()),
            king_keys: core::array::from_fn(|_| rng.next_u64()),
            enpassant_keys: core::array::from_fn(|_| rng.next_u64()),
            kingside_key: rng.next_u64(),
            queenside_key: rng.next_u64(),
        }
    }
}

impl ZobristKeys {
    pub fn new() -> Self {
        let mut rng = ChaCha20Rng::from_seed([42; 32]);
        
        Self { white_keys: SideZobristKeys::new(&mut rng),
               black_keys: SideZobristKeys::new(&mut rng)
        }
    }
}

#[derive(Clone, Copy)]
pub enum Flag {
    EXACT,
    LOWERBOUND,
    UPPERBOUND,
}

pub struct TTable {
    table: Vec<TTVal>
}

#[derive(Clone, Copy)]
pub struct TTVal {
    pub flag: Flag,
    pub depth: u8,
    pub value: i32,
    pub full_hash: u64,
}

impl TTable {
    pub fn new(table_bits: u8) -> Self {
        let default = TTVal{ flag: Flag::EXACT, depth: 0, value: 0, full_hash: 0 };
        Self { table: vec![default; 1 << table_bits] }
    }

    pub fn insert(bitboard: &mut BitBoardGame, flag: Flag, depth: u8, value: i32, full_hash: u64) {
        
    }
}

fn main() {
    let hello = ZobristKeys::new();
    print!("{:#?}", hello);
    let mut node = BitBoardGame::from_fen("kbK5/pp6/1P6/8/8/8/8/R7 w - - 0 1").unwrap();



    println!("{}", best_move(&mut node, 7, 1).to_uci());
}

fn best_move(node: &mut BitBoardGame, depth: u8, turn: i32) -> u16 {
    let mut best_val = -i32::MAX;
    let mut best_move: u16 = 0;

    for mov in node.moves() {
        let mut new_node = node.clone();
        new_node.do_move(&mov);
        let value = -negamax(&mut new_node, depth - 1, i32::MAX, -i32::MAX, -turn);
        if value >= best_val {
            best_val = value;
            best_move = mov.mov;
        }
    }

    best_move
}

fn negamax(node: &mut BitBoardGame, depth: u8, a: i32, b: i32, turn: i32) -> i32 {
    if depth == 0 {
        return turn * eval(node);
    }

    let moves = order_moves(&node.moves());

    if moves.is_empty() && node.board.check_mask(turn == 1) != 0 {
        return turn * i32::MAX;
    }

    let mut value = -i32::MAX;
    for mov in moves {
        let mut new_node = node.clone();
        new_node.do_move(&mov);
        value = max(value, -negamax(&mut new_node, depth - 1, -b, -a, -turn));
        let a_new = max(a, value);
        if a_new >= b {
            break;
        }
    }

    value
}

fn order_moves(moves: &Vec<BitBoardGameMove>) -> Vec<BitBoardGameMove> {
    let mut new_moves: Vec<BitBoardGameMove> = Vec::new();
    for mov in moves {
        new_moves.push(mov.clone());
    }
    new_moves
}

fn eval(node: &BitBoardGame) -> i32 {
    (node.board.col_pawn_mask(true).count_ones()
        + node.board.col_knight_mask(true).count_ones() * 3
        + node.board.col_diagonal_mask(true).count_ones() * 3
        + node.board.col_ortho_mask(true).count_ones() * 5
        + node.board.col_king_mask(true).count_ones() * 50) as i32
        - (node.board.col_pawn_mask(false).count_ones()
            + node.board.col_knight_mask(false).count_ones() * 3
            + node.board.col_diagonal_mask(false).count_ones() * 3
            + node.board.col_ortho_mask(false).count_ones() * 5
            + node.board.col_king_mask(false).count_ones() * 50) as i32
}

// function init_zobrist():
//     # fill a table of random numbers/bitstrings
//     table := a 2-d array of size 64×12
//     for i from 1 to 64:  # loop over the board, represented as a linear array
//         for j from 1 to 12:      # loop over the pieces
//             table[i][j] := random_bitstring()
//     table.black_to_move = random_bitstring()

// function hash(board):
//     h := 0
//     if is_black_turn(board):
//         h := h XOR table.black_to_move
//     for i from 1 to 64:      # loop over the board positions
//         if board[i] ≠ empty:
//             j := the piece at board[i], as listed in the constant indices, above
//             h := h XOR table[i][j]
//     return h
