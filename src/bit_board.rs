use std::fmt::Display;

use crate::{notation::AlgebraicMove, game::{ChessGame, Move}, piece::PlayerColour};

struct BBMove {
    /// 0b-pccvvvuuuyyyxxx
    /// xxx from x
    /// yyy from y
    /// uuu to x
    /// vvv from y
    /// cc promotion
    ///   00 = knight
    ///   01 = bishop
    ///   10 = rook
    ///   11 = queen
    /// p promote
    packed: u16
}

/// Column wise repr of chess board
/// 0000 => none
/// 1000 => enpasentable pawn
/// 1??? => white
/// 0??? => black
/// ?001 => bishop
/// ?010 => rook
/// ?011 => queen
/// ?100 => pawn
/// ?101 => knight
/// ?110 => castleable rook
/// ?111 => king

struct BitBoard {
    board: [u64; 4]
}

impl BitBoard {

    /// set cell `square` to an empty cell
    pub fn clear(&mut self, square: u8) {
        for line in self.board.iter_mut() {
            *line &= !(1 << square);
        }
    }

    /// copy cell `old` to cell `new`
    pub fn dupe(&mut self, old: u8, new: u8) {
        for line in self.board.iter_mut() {
            let tmp = (*line >> old) & 1;
            *line &= !(1 << new);
            *line |= !(tmp << new);
        }
    }

    /// 1 if white
    /// 0 if black or no piece
    pub const fn colour_mask(&self) -> u64 {
        self.board[3]
    }

    /// 1 if real (exc enpasentable pawns) piece
    /// 0 if no piece
    pub const fn piece_mask(&self) -> u64 {
        self.board[0] | self.board[1] | self.board[2]
    }

    /// 1 if real (exc enpasentable pawns) white piece
    /// 0 if no piece
    pub const fn white_piece_mask(&self) -> u64 {
        self.pawn_mask() & self.colour_mask()
    }

    /// 1 if real (exc enpasentable pawns) black piece
    /// 0 if no piece
    pub const fn black_piece_mask(&self) -> u64 {
        self.pawn_mask() & !self.colour_mask()
    }

    /// 1 if real (exc enpasentable pawns) black piece
    /// 0 if no piece
    pub const fn side_piece_mask(&self, side: u64) -> u64 {
        self.pawn_mask() & (self.colour_mask() ^ side)
    }

    /// 1 if piece inc special
    /// 0 if no piece
    pub const fn piece_special_mask(&self) -> u64 {
        self.board[0] | self.board[1] | self.board[2] | self.board[3]
    }

    /// 1 if pawn
    /// 0 if no pawn
    pub const fn pawn_mask(&self) -> u64 {
        !self.board[0] & !self.board[1] & self.board[2]
    }

    /// 1 if white pawn
    /// 0 if no white pawn
    pub const fn white_pawn_mask(&self) -> u64 {
        self.pawn_mask() & self.colour_mask()
    }

    /// 1 if black pawn
    /// 0 if no black pawn
    pub const fn black_pawn_mask(&self) -> u64 {
        self.pawn_mask() & !self.colour_mask()
    }

    /// side 0 = white, u64::MAX = black
    /// 1 if side pawn
    /// 0 if no side pawn
    pub const fn side_pawn_mask(&self, side: u64) -> u64 {
        self.pawn_mask() & (self.colour_mask() ^ side)
    }

    /// 1 if knight
    /// 0 if no knight
    pub const fn knight_mask(&self) -> u64 {
        self.board[0] & !self.board[1] & self.board[2]
    }

    /// 1 if white knight
    /// 0 if no white knight
    pub const fn white_knight_mask(&self) -> u64 {
        self.knight_mask() & self.colour_mask()
    }

    /// 1 if black knight
    /// 0 if no black knight
    pub const fn black_knight_mask(&self) -> u64 {
        self.knight_mask() & !self.colour_mask()
    }

    /// side 0 = white, u64::MAX = black
    /// 1 if side knight
    /// 0 if no side knight
    pub const fn side_knight_mask(&self, side: u64) -> u64 {
        self.knight_mask() & (self.colour_mask() ^ side)
    }

    /// side 0 = white, u64::MAX = black
    /// 1 if side knight
    /// 0 if no side knight
    pub const fn knight_attack_mask(&self, side: u64) -> u64 {
        let knights = self.side_knight_mask(side);
        let movable_squares = !self.side_piece_mask(side);
        //0b11111100
        let knights_r2 = knights & 0xfcfcfcfcfcfcfc;
        //0b11111110
        let knights_r1 = knights & 0xfefefefefefefe;
        //0b11111100
        let knights_l1 = knights & 0x7f7f7f7f7f7f7f;
        //0b11111110
        let knights_l2 = knights & 0x3f3f3f3f3f3f3f;
        let potential_moves = (knights_r2 >> 10) | (knights_r2 << 6) |
            (knights_r1 >> 17) | (knights_r1 << 15) |
            (knights_l1 >> 15) | (knights_l1 << 17) |
            (knights_l2 >> 6) | (knights_l2 << 10);
        potential_moves & movable_squares
    }
}

struct BitBoardGame {
    board: BitBoard,
    turn: PlayerColour,
}

impl ChessGame for BitBoardGame {
    type Move = BBMove;

    type UnMove = BitBoard;

    fn new() -> Self {
        todo!()
    }

    fn from_fen(fen: &str) -> Option<Self> {
        todo!()
    }

    fn decode_alg(&mut self, mov: &AlgebraicMove) -> Self::Move {
        todo!()
    }

    fn moves(&mut self) -> Vec<Self::Move> {
        let side_mask = match self.turn {
            PlayerColour::White => 0u64,
            PlayerColour::Black => u64::MAX,
        };
        todo!()
    }

    fn do_move(&mut self, mov: &Self::Move) -> Self::UnMove {
        todo!()
    }

    fn unmove(&mut self, mov: &Self::UnMove) {
        todo!()
    }

    fn gen_alg(&mut self, mov: &Self::Move) -> AlgebraicMove {
        todo!()
    }
}

impl Display for BBMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.packed))
    }
}

impl Move for BBMove {
    fn to_uci(&self) -> String {
        let ox = ('a' as u8 + (self.packed & 7) as u8) as char;
        let oy = ('1' as u8 + ((self.packed >> 3) & 7) as u8) as char;
        let nx = ('a' as u8 + ((self.packed >> 6) & 7) as u8) as char;
        let ny = ('1' as u8 + ((self.packed >> 9) & 7) as u8) as char;
        format!("{ox}{oy}{nx}{ny}")
    }
}