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

pub struct BitBoard {
    board: [u64; 4]
}

impl BitBoard {
    const RIGHT_SIDE: u64 = 0x0101010101010101;
    const LEFT_SIDE: u64 = 0x8080808080808080;
    const RIGHT2_SIDE: u64 = Self::RIGHT_SIDE | (Self::RIGHT_SIDE << 1);
    const LEFT2_SIDE: u64 = Self::LEFT_SIDE | (Self::LEFT_SIDE << 1);

    /// set cell `square` to an empty cell
    #[inline(always)]
    pub fn clear(&mut self, square: u8) {
        for line in self.board.iter_mut() {
            *line &= !(1 << square);
        }
    }

    /// copy cell `old` to cell `new`
    #[inline(always)]
    pub fn dupe(&mut self, old: u8, new: u8) {
        for line in self.board.iter_mut() {
            let tmp = (*line >> old) & 1;
            *line &= !(1 << new);
            *line |= !(tmp << new);
        }
    }

    /// 1 if white
    /// 0 if black or no piece
    #[inline(always)]
    pub const fn colour_mask(&self) -> u64 {
        self.board[3]
    }

    /// 1 if real (exc enpasentable pawns) piece
    /// 0 if no piece
    #[inline(always)]
    pub const fn piece_mask(&self) -> u64 {
        self.board[0] | self.board[1] | self.board[2]
    }

    /// 1 if real (exc enpasentable pawns) black piece
    /// 0 if no piece
    #[inline(always)]
    pub const fn side_piece_mask(&self, side: u64) -> u64 {
        self.pawn_mask() & (self.colour_mask() ^ side)
    }

    /// 1 if piece inc special
    /// 0 if no piece
    #[inline(always)]
    pub const fn piece_special_mask(&self) -> u64 {
        self.board[0] | self.board[1] | self.board[2] | self.board[3]
    }

    /// 1 if pawn
    /// 0 if no pawn
    #[inline(always)]
    pub const fn pawn_mask(&self) -> u64 {
        !self.board[0] & !self.board[1] & self.board[2]
    }

    /// side 0 = white, u64::MAX = black
    /// 1 if side pawn
    /// 0 if no side pawn
    #[inline(always)]
    pub const fn side_pawn_mask(&self, side: u64) -> u64 {
        self.pawn_mask() & (self.colour_mask() ^ side)
    }

    /// side 0 = white, u64::MAX = black
    /// 1 if side pawn
    /// 0 if no side pawn
    #[inline(always)]
    pub const fn pawn_attack_mask(&self, side: u64) -> u64 {
        let pawns = self.side_pawn_mask(side);
        let left_ls = 9 & !side;
        let left_rs = 7 & side;
        let right_ls = 7 & !side;
        let right_rs = 9 & side;
        let left_pawns = (pawns << left_ls) | (pawns >> left_rs) & !Self::RIGHT_SIDE;
        let right_pawns = (pawns << right_ls) | (pawns >> right_rs) & !Self::LEFT_SIDE;
        left_pawns | right_pawns
    }

    /// 1 if knight
    /// 0 if no knight
    #[inline(always)]
    pub const fn knight_mask(&self) -> u64 {
        self.board[0] & !self.board[1] & self.board[2]
    }

    /// side 0 = white, u64::MAX = black
    /// 1 if side knight
    /// 0 if no side knight
    #[inline(always)]
    pub const fn side_knight_mask(&self, side: u64) -> u64 {
        self.knight_mask() & (self.colour_mask() ^ side)
    }

    /// side 0 = white, u64::MAX = black
    /// 1 if side knight can attack
    /// 0 if no side knight cant attack
    #[inline(always)]
    pub const fn knight_attack_mask(&self, side: u64) -> u64 {
        let knights = self.side_knight_mask(side);
        //0b11111100
        let knights_r2 = knights & !Self::LEFT2_SIDE;
        //0b11111110
        let knights_r1 = knights & !Self::LEFT_SIDE;
        //0b11111100
        let knights_l1 = knights & !Self::RIGHT_SIDE;
        //0b11111110
        let knights_l2 = knights & !Self::RIGHT2_SIDE;
        (knights_r2 >> 10) | (knights_r2 << 6) |
            (knights_r1 >> 17) | (knights_r1 << 15) |
            (knights_l1 >> 15) | (knights_l1 << 17) |
            (knights_l2 >> 6) | (knights_l2 << 10)
    }

    /// 1 if bishop like
    /// 0 if no bishop like
    #[inline(always)]
    pub const fn bishop_like_mask(&self) -> u64 {
        self.board[0] & !self.board[2]
    }

    /// side 0 = white, u64::MAX = black
    /// 1 if side bishop like
    /// 0 if no side bishop like
    #[inline(always)]
    pub const fn side_bishop_like_mask(&self, side: u64) -> u64 {
        self.bishop_like_mask() & (self.colour_mask() ^ side)
    }

    /// side 0 = white, u64::MAX = black
    /// 1 if side bishop can attack
    /// 0 if no side bishop cant attack
    /// Note: a queen is a bishop
    #[inline(always)]
    pub const fn bishop_like_attack_mask(&self, side: u64) -> u64 {
        let bishops = self.side_bishop_like_mask(side);
        let pieces = self.piece_mask();
        let mut ur = (bishops << 7) & !Self::LEFT_SIDE;
        let mut ul = (bishops << 9) & !Self::RIGHT_SIDE;
        let mut dr = (bishops >> 9) & !Self::LEFT_SIDE;
        let mut dl = (bishops >> 7) & !Self::RIGHT_SIDE;
        let mut i = 0;
        while i != 6 {
            ur |= ((ur & !pieces) << 7) & !Self::LEFT_SIDE;
            ul |= ((ul & !pieces) << 9) & !Self::RIGHT_SIDE;
            dr |= ((dr & !pieces) >> 9) & !Self::LEFT_SIDE;
            dl |= ((dl & !pieces) >> 7) & !Self::RIGHT_SIDE;
            i += 1;
        }
        ur | ul | dr | dl
    }

    /// 1 if rook like
    /// 0 if no rook like
    #[inline(always)]
    pub const fn rook_like_mask(&self) -> u64 {
        self.board[1] & !(self.board[0] & self.board[2])
    }

    /// side 0 = white, u64::MAX = black
    /// 1 if side rook like
    /// 0 if no side rook like
    #[inline(always)]
    pub const fn side_rook_like_mask(&self, side: u64) -> u64 {
        self.rook_like_mask() & (self.colour_mask() ^ side)
    }

    /// side 0 = white, u64::MAX = black
    /// 1 if side rook can attack
    /// 0 if no side rook cant attack
    /// Note: a queen is a rook
    #[inline(always)]
    pub const fn rook_like_attack_mask(&self, side: u64) -> u64 {
        let rooks = self.side_rook_like_mask(side);
        let pieces = self.piece_mask();
        let mut r = (rooks >> 1) & !Self::LEFT_SIDE;
        let mut l = (rooks << 1) & !Self::RIGHT_SIDE;
        let mut u = rooks << 8;
        let mut d = rooks >> 8;
        let mut i = 0;
        while i != 6 {
            r |= ((r & !pieces) >> 1) & !Self::LEFT_SIDE;
            l |= ((l & !pieces) << 1) & !Self::RIGHT_SIDE;
            u |= (u & !pieces) << 8;
            d |= (d & !pieces) >> 8;
            i += 1;
        }
        r | l | u | d
    }

    /// 1 if king
    /// 0 if no king
    #[inline(always)]
    pub const fn king_mask(&self) -> u64 {
        self.board[0] & self.board[1] & self.board[2]
    }

    /// side 0 = white, u64::MAX = black
    /// 1 if side king
    /// 0 if no side king
    #[inline(always)]
    pub const fn side_king_mask(&self, side: u64) -> u64 {
        self.king_mask() & (self.colour_mask() ^ side)
    }

    /// side 0 = white, u64::MAX = black
    /// 1 if side king can attack
    /// 0 if no side king cant attack
    #[inline(always)]
    pub const fn king_attack_mask(&self, side: u64) -> u64 {
        let kings = self.side_king_mask(side);
        let u = kings << 8;
        let d = kings >> 8;
        let mast =  kings | u | d;
        ((mast >> 1) & !Self::LEFT_SIDE) | ((mast << 1) & !Self::RIGHT_SIDE) | u | d
    }

    #[inline(always)]
    pub const fn attack_mask(&self, side: u64) -> u64 {
        self.pawn_attack_mask(side) |
            self.knight_attack_mask(side) |
            self.bishop_like_attack_mask(side) |
            self.rook_like_attack_mask(side) |
            self.king_attack_mask(side)
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