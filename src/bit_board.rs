use std::{fmt::Display, marker::ConstParamTy};

use crate::{notation::AlgebraicMove, game::{ChessGame, Move}, piece::PlayerColour};

pub struct BBMove {
    /// 0b-pccvvvuuuyyyxxx
    /// xxx: from x
    /// yyy: from y
    /// uuu: to x
    /// vvv: to y
    /// cc: promotion type
    ///  - 00 = knight
    ///  - 01 = bishop
    ///  - 10 = rook
    ///  - 11 = queen
    /// p: promotion flag
    packed: u16
}

pub struct BoolExists<const _N: bool>{}

/// Column-wise representation of chess board (if you stack each u64 on top of each other)
/// 0000 => none
/// 1000 => enpassant square
/// 1??? => white
/// 0??? => black
/// ?001 => bishop
/// ?010 => rook
/// ?011 => queen
/// ?100 => pawn
/// ?101 => knight
/// ?110 => castleable rook
/// ?111 => king
#[derive(Clone)]
pub struct BitBoard {
    // Index that corresponds to each bit: 0b3210
    board: [u64; 4]
}

#[derive(ConstParamTy, PartialEq, Eq)]
pub enum Shift {
    Left,
    Right
}

pub trait OnMove {
    fn on_move<const TURN: bool, const WQ: bool,
        const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard, from: u8, to: u8);
    fn on_rook_move<const TURN: bool, const WQ: bool,
        const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard, from: u8, to: u8);
    fn on_king_move<const TURN: bool, const WQ: bool,
        const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard, from: u8, to: u8);
    fn on_ep_move<const TURN: bool, const WQ: bool,
        const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard, from: u8, to: u8);
    fn on_pawn_push2<const TURN: bool, const WQ: bool,
        const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard, from: u8);
    fn on_qs_castle<const TURN: bool, const WQ: bool,
        const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard);
    fn on_ks_castle<const TURN: bool, const WQ: bool,
        const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard);
}

impl BitBoard {
    const LEFT_SIDE: u64 = 0x8080808080808080;
    const RIGHT_SIDE: u64 = 0x0101010101010101;
    const LEFT2_SIDE: u64 = Self::LEFT_SIDE | (Self::LEFT_SIDE >> 1);
    const RIGHT2_SIDE: u64 = Self::RIGHT_SIDE | (Self::RIGHT_SIDE << 1);

    /// set cell `square` to an empty cell
    #[inline(always)]
    pub fn clear(&mut self, square: u8) {
        for line in self.board.iter_mut() {
            *line &= !(1 << square);
        }
    }

    /// copy cell `from` to cell `to`
    #[inline(always)]
    pub fn dupe(&mut self, from: u8, to: u8) {
        for line in self.board.iter_mut() {
            let tmp = (*line >> from) & 1;
            *line &= !(1 << to);
            *line |= !(tmp << to);
        }
    }

    /// move piece
    #[inline(always)]
    pub fn mov(&mut self, from: u8, to: u8) {
        self.dupe(from, to);
        self.clear(from);
    }

    /// 1 if white
    /// 0 if black or no piece
    #[inline(always)]
    pub const fn colour_mask<const COLOUR: bool>(&self) -> u64 {
        if COLOUR {
            self.board[3]
        }
        else {
            !self.board[3]
        }
    }

    /// 1 if real piece (excl. enpassantable pawns)
    /// 0 if no piece
    #[inline(always)]
    pub const fn piece_mask(&self) -> u64 {
        self.board[0] | self.board[1] | self.board[2]
    }

    /// 1 if real white/black piece (excl. enpassantable pawns)
    /// 0 if not white/black piece
    #[inline(always)]
    pub const fn col_piece_mask<const COLOUR: bool>(&self) -> u64 {
        self.piece_mask() & self.colour_mask::<COLOUR>()
    }

    #[inline(always)]
    pub const fn enemy_or_empty<const COLOUR: bool>(&self) -> u64 {
        !self.col_piece_mask::<COLOUR>()
    }

    /// 1 if piece (incl. special enpassant square)
    /// 0 if no piece
    #[inline(always)]
    pub const fn piece_special_mask(&self) -> u64 {
        self.board[0] | self.board[1] | self.board[2] | self.board[3]
    }

    #[inline(always)]
    pub const fn sliding_mask<const SHIFT_DIR: Shift>(pieces: u64, step: u8, colision: u64, side_mask: u64) -> u64 {
        let mut mask = match SHIFT_DIR {
            Shift::Left => (pieces << step) & !side_mask,
            Shift::Right => (pieces >> step) & !side_mask,
        };
        let mut i = 0;
        while i != 6 {
            mask |= match SHIFT_DIR {
                Shift::Left => ((mask & !colision) << step) & !side_mask,
                Shift::Right => ((mask & !colision) >> step) & !side_mask,
            };
            i += 1;
        }
        mask
    }

    /// 1 if pawn
    /// 0 if no pawn
    #[inline(always)]
    pub const fn pawn_mask(&self) -> u64 {
        !self.board[0] & !self.board[1] & self.board[2]
    }

    /// colour 0 = white, u64::MAX = black
    /// 1 if colour pawn
    /// 0 if no colour pawn
    #[inline(always)]
    pub const fn col_pawn_mask<const COLOUR: bool>(&self) -> u64 {
        self.pawn_mask() & self.colour_mask::<COLOUR>()
    }

    /// colour 0 = white, u64::MAX = black
    /// 1 if colour pawn
    /// 0 if no colour pawn
    #[inline(always)]
    pub const fn pawn_attack_mask<const COLOUR: bool>(&self) -> u64 {
        let pawns = self.col_pawn_mask::<COLOUR>();
        if COLOUR {
            ((pawns << 9) & !Self::RIGHT_SIDE) |
            ((pawns << 7) & !Self::LEFT_SIDE)
        }
        else {
            ((pawns >> 7) & !Self::RIGHT_SIDE) |
            ((pawns >> 9) & !Self::LEFT_SIDE)
        }
    }

    #[inline(always)]
    pub const fn pawn_like_attack_mask<const COLOUR: bool>(&self, pieces: u64) -> u64 {
        if COLOUR {
            ((pieces << 9) & !Self::RIGHT_SIDE) |
            ((pieces << 7) & !Self::LEFT_SIDE)
        }
        else {
            ((pieces >> 7) & !Self::RIGHT_SIDE) |
            ((pieces >> 9) & !Self::LEFT_SIDE)
        }
    }

    #[inline(always)]
    pub const fn pawn_move_mask<const COLOUR: bool>(&self) -> u64 {
        let pawns = self.col_pawn_mask::<COLOUR>();
        let pieces = self.piece_mask();

        if COLOUR {
            let step = (pawns << 8) & !pieces;
            step | ((step << 8) & !pieces & 0xff000000)
        } else {
            let step = (pawns >> 8) & !pieces;
            step | ((step >> 8) & !pieces & 0xff00000000)
        }
    }

    #[inline(always)]
    pub const fn pawn_like_move_mask<const COLOUR: bool>(&self, pieces: u64) -> u64 {
        let blockers = self.piece_mask();

        if COLOUR {
            let step = (pieces << 8) & !blockers;
            step | ((step << 8) & !blockers & 0xff000000)
        } else {
            let step = (pieces >> 8) & !blockers;
            step | ((step >> 8) & !blockers & 0xff00000000)
        }
    }

    /// 1 if knight
    /// 0 if no knight
    #[inline(always)]
    pub const fn knight_mask(&self) -> u64 {
        self.board[0] & !self.board[1] & self.board[2]
    }

    /// colour 0 = white, u64::MAX = black
    /// 1 if colour knight
    /// 0 if no colour knight
    #[inline(always)]
    pub const fn col_knight_mask<const COLOUR: bool>(&self) -> u64 {
        self.knight_mask() & (self.colour_mask::<COLOUR>())
    }

    /// colour 0 = white, u64::MAX = black
    /// 1 if colour knight can attack
    /// 0 if no colour knight cant attack
    #[inline(always)]
    pub const fn knight_attack_mask<const COLOUR: bool>(&self) -> u64 {
        let knights = self.col_knight_mask::<COLOUR>();
        //0b11111100
        let knights_r2 = knights & !Self::RIGHT2_SIDE;
        //0b11111110
        let knights_r1 = knights & !Self::RIGHT_SIDE;
        //0b11111100
        let knights_l1 = knights & !Self::LEFT_SIDE;
        //0b11111110
        let knights_l2 = knights & !Self::LEFT2_SIDE;
        let inner = (knights_r1 >> 1) | (knights_l1 << 1);
        let outer = (knights_r2 >> 2) | (knights_l2 << 2);
        (outer << 8) | (outer >> 8) | (inner << 16) | (inner >> 16)
    }

    #[inline(always)]
    pub const fn knight_like_attack_mask(&self, pieces: u64) -> u64 {
        //0b11111100
        let knights_r2 = pieces & !Self::RIGHT2_SIDE;
        //0b11111110
        let knights_r1 = pieces & !Self::RIGHT_SIDE;
        //0b11111100
        let knights_l1 = pieces & !Self::LEFT_SIDE;
        //0b11111110
        let knights_l2 = pieces & !Self::LEFT2_SIDE;
        let inner = (knights_r1 >> 1) | (knights_l1 << 1);
        let outer = (knights_r2 >> 2) | (knights_l2 << 2);
        (outer << 8) | (outer >> 8) | (inner << 16) | (inner >> 16)
    }

    /// 1 if bishop like
    /// 0 if no bishop like
    #[inline(always)]
    pub const fn diagonal_mask(&self) -> u64 {
        self.board[0] & !self.board[2]
    }

    /// colour 0 = white, u64::MAX = black
    /// 1 if colour bishop like
    /// 0 if no colour bishop like
    #[inline(always)]
    pub const fn col_diagonal_mask<const COLOUR: bool>(&self) -> u64 {
        self.diagonal_mask() & (self.colour_mask::<COLOUR>())
    }

    /// colour 0 = white, u64::MAX = black
    /// 1 if colour bishop can attack
    /// 0 if no colour bishop cant attack
    /// Note: a queen is a bishop
    #[inline(always)]
    pub const fn diagonal_attack_mask<const COLOUR: bool>(&self) -> u64 {
        let bishops = self.col_diagonal_mask::<COLOUR>();
        let pieces = self.piece_mask();
        let ur = Self::sliding_mask::<{Shift::Left}>(bishops, 7, pieces, Self::LEFT_SIDE);
        let ul = Self::sliding_mask::<{Shift::Left}>(bishops, 9, pieces, Self::RIGHT_SIDE);
        let dr = Self::sliding_mask::<{Shift::Right}>(bishops, 9, pieces, Self::LEFT_SIDE);
        let dl = Self::sliding_mask::<{Shift::Right}>(bishops, 7, pieces, Self::RIGHT_SIDE);
        ur | ul | dr | dl
    }

    #[inline(always)]
    pub const fn diagonal_like_attack_mask(&self, pieces: u64) -> u64 {
        let blockers = self.piece_mask();
        let ur = Self::sliding_mask::<{Shift::Left}>(pieces, 7, blockers, Self::LEFT_SIDE);
        let ul = Self::sliding_mask::<{Shift::Left}>(pieces, 9, blockers, Self::RIGHT_SIDE);
        let dr = Self::sliding_mask::<{Shift::Right}>(pieces, 9, blockers, Self::LEFT_SIDE);
        let dl = Self::sliding_mask::<{Shift::Right}>(pieces, 7, blockers, Self::RIGHT_SIDE);
        ur | ul | dr | dl
    }

    /// 1 if rook like
    /// 0 if no rook like
    #[inline(always)]
    pub const fn ortho_mask(&self) -> u64 {
        self.board[1] & !(self.board[0] & self.board[2])
    }

    /// colour 0 = white, u64::MAX = black
    /// 1 if colour rook like
    /// 0 if no colour rook like
    #[inline(always)]
    pub const fn col_ortho_mask<const COLOUR: bool>(&self) -> u64 {
        self.ortho_mask() & (self.colour_mask::<COLOUR>())
    }

    /// colour 0 = white, u64::MAX = black
    /// 1 if colour rook can attack
    /// 0 if no colour rook cant attack
    /// Note: a queen is a rook
    #[inline(always)]
    pub const fn ortho_attack_mask<const COLOUR: bool>(&self) -> u64 {
        let rooks = self.col_ortho_mask::<COLOUR>();
        let pieces = self.piece_mask();
        let r = Self::sliding_mask::<{Shift::Left}>(rooks, 1, pieces, Self::RIGHT_SIDE);
        let l = Self::sliding_mask::<{Shift::Right}>(rooks, 1, pieces, Self::LEFT_SIDE);
        let u = Self::sliding_mask::<{Shift::Left}>(rooks, 8, pieces, 0);
        let d = Self::sliding_mask::<{Shift::Right}>(rooks, 8, pieces, 0);
        r | l | u | d
    }

    #[inline(always)]
    pub const fn ortho_like_attack_mask(&self, pieces: u64) -> u64 {
        let blockers = self.piece_mask();
        let r = Self::sliding_mask::<{Shift::Left}>(pieces, 1, blockers, Self::RIGHT_SIDE);
        let l = Self::sliding_mask::<{Shift::Right}>(pieces, 1, blockers, Self::LEFT_SIDE);
        let u = Self::sliding_mask::<{Shift::Left}>(pieces, 8, blockers, 0);
        let d = Self::sliding_mask::<{Shift::Right}>(pieces, 8, blockers, 0);
        r | l | u | d
    }


    /// 1 if king
    /// 0 if no king
    #[inline(always)]
    pub const fn king_mask(&self) -> u64 {
        self.board[0] & self.board[1] & self.board[2]
    }

    /// colour 0 = white, u64::MAX = black
    /// 1 if colour king
    /// 0 if no colour king
    #[inline(always)]
    pub const fn col_king_mask<const COLOUR: bool>(&self) -> u64 {
        self.king_mask() & (self.colour_mask::<COLOUR>())
    }

    /// colour 0 = white, u64::MAX = black
    /// 1 if colour king can attack
    /// 0 if no colour king cant attack
    #[inline(always)]
    pub const fn king_attack_mask<const COLOUR: bool>(&self) -> u64 {
        let kings = self.col_king_mask::<COLOUR>();
        let u = kings << 8;
        let d = kings >> 8;
        let mast =  kings | u | d;
        ((mast >> 1) & !Self::LEFT_SIDE) | ((mast << 1) & !Self::RIGHT_SIDE) | u | d
    }

    #[inline(always)]
    pub const fn attack_mask<const COLOUR: bool>(&self) -> u64 {
        self.pawn_attack_mask::<COLOUR>() |
            self.knight_attack_mask::<COLOUR>() |
            self.diagonal_attack_mask::<COLOUR>() |
            self.ortho_attack_mask::<COLOUR>() |
            self.king_attack_mask::<COLOUR>()
    }

    #[inline(always)]
    pub const fn check_mask<const COLOUR: bool>(&self) -> u64
    where BoolExists<{!COLOUR}>: Sized {
        let kings = self.col_king_mask::<COLOUR>();
        let pieces = self.piece_mask();
        let other_ortho = self.col_ortho_mask::<{!COLOUR}>();
        let other_diag = self.col_diagonal_mask::<{!COLOUR}>();
        let mut mask = u64::MAX;
        let r1 = Self::sliding_mask::<{Shift::Right}>(kings, 1, pieces, Self::LEFT_SIDE);
        if r1 & other_ortho != 0 {
            mask &= r1;
        }

        let r1 = Self::sliding_mask::<{Shift::Left}>(kings, 1, pieces, Self::RIGHT_SIDE);
        if r1 & other_ortho != 0 {
            mask &= r1;
        }

        let r1 = Self::sliding_mask::<{Shift::Left}>(kings, 8, pieces, 0);
        if r1 & other_ortho != 0 {
            mask &= r1;
        }

        let r1 = Self::sliding_mask::<{Shift::Right}>(kings, 8, pieces, 0);
        if r1 & other_ortho != 0 {
            mask &= r1;
        }

        let r1 = Self::sliding_mask::<{Shift::Right}>(kings, 9, pieces, Self::LEFT_SIDE);
        if r1 & other_diag != 0 {
            mask &= r1;
        }

        let r1 = Self::sliding_mask::<{Shift::Left}>(kings, 9, pieces, Self::RIGHT_SIDE);
        if r1 & other_diag != 0 {
            mask &= r1;
        }

        let r1 = Self::sliding_mask::<{Shift::Left}>(kings, 7, pieces, Self::LEFT_SIDE);
        if r1 & other_diag != 0 {
            mask &= r1;
        }

        let r1 = Self::sliding_mask::<{Shift::Right}>(kings, 7, pieces, Self::RIGHT_SIDE);
        if r1 & other_diag != 0 {
            mask &= r1;
        }

        let other_knights = Self::col_knight_mask::<{!COLOUR}>(self);
        let knights_r2 = kings & !Self::RIGHT2_SIDE;
        let knights_r1 = kings & !Self::RIGHT_SIDE;
        let knights_l1 = kings & !Self::LEFT_SIDE;
        let knights_l2 = kings & !Self::LEFT2_SIDE;

        if (knights_r2 >> 10) & other_knights != 0 {
            mask &= knights_r2 >> 10;
        }
        if (knights_r2 << 6) & other_knights != 0 {
            mask &= knights_r2 << 6;
        }
        if (knights_r1 >> 17) & other_knights != 0 {
            mask &= knights_r2 >> 17;
        }
        if (knights_r1 << 15) & other_knights != 0 {
            mask &= knights_r2 << 15;
        }
        if (knights_l1 >> 15) & other_knights != 0 {
            mask &= knights_r2 >> 15;
        }
        if (knights_l1 << 17) & other_knights != 0 {
            mask &= knights_r2 << 17;
        }
        if (knights_l2 >> 6) & other_knights != 0 {
            mask &= knights_r2 >> 6;
        }
        if (knights_l2 << 10) & other_knights != 0 {
            mask &= knights_r2 << 10;
        }

        let other_pawns = Self::col_pawn_mask::<{!COLOUR}>(self);

        if COLOUR {
            if (kings << 9) & !Self::RIGHT_SIDE & other_pawns != 0 {
                mask &= kings << 9;
            }
            if (kings << 7) & !Self::LEFT_SIDE & other_pawns != 0 {
                mask &= kings << 7;
            }
        } else {
            if (kings >> 7) & !Self::RIGHT_SIDE & other_pawns != 0 {
                mask &= kings >> 7;
            }
            if (kings >> 9) & !Self::LEFT_SIDE & other_pawns != 0 {
                mask &= kings >> 9;
            }
        }

        mask
    }

    #[inline(always)]
    pub const fn vert_pin_mask<const COLOUR: bool>(&self) -> u64
    where BoolExists<{!COLOUR}>: Sized {
        let kings = self.col_king_mask::<COLOUR>();
        let pieces = self.piece_mask();
        //let own_pieces = self.col_piece_mask::<COLOUR>();
        let other_ortho = self.col_ortho_mask::<{!COLOUR}>();
        let mut mask = 0;
        let r1 = Self::sliding_mask::<{Shift::Left}>(kings, 8, pieces, 0);
        let r2 = Self::sliding_mask::<{Shift::Left}>(r1 & pieces, 8, pieces, 0);
        if r2 & other_ortho != 0 {
            mask |= r1 | r2;
        }

        let r1 = Self::sliding_mask::<{Shift::Right}>(kings, 8, pieces, 0);
        let r2 = Self::sliding_mask::<{Shift::Right}>(r1 & pieces, 8, pieces, 0);
        if r2 & other_ortho != 0 {
            mask |= r1 | r2;
        }

        mask
    }

    #[inline(always)]
    pub const fn hor_pin_mask<const COLOUR: bool>(&self) -> u64
    where BoolExists<{!COLOUR}>: Sized {
        let kings = self.col_king_mask::<COLOUR>();
        let pieces = self.piece_mask();
        //let own_pieces = self.col_piece_mask::<COLOUR>();
        let other_ortho = self.col_ortho_mask::<{!COLOUR}>();
        let r1 = Self::sliding_mask::<{Shift::Right}>(kings, 1, pieces, Self::LEFT_SIDE);
        let r2 = Self::sliding_mask::<{Shift::Right}>(r1 & pieces, 1, pieces, Self::LEFT_SIDE);
        let mut mask = 0;
        if r2 & other_ortho != 0 {
            mask |= r1 | r2;
        }

        let r1 = Self::sliding_mask::<{Shift::Left}>(kings, 1, pieces, Self::RIGHT_SIDE);
        let r2 = Self::sliding_mask::<{Shift::Left}>(r1 & pieces, 1, pieces, Self::RIGHT_SIDE);
        if r2 & other_ortho != 0 {
            mask |= r1 | r2;
        }

        mask
    }

    #[inline(always)]
    pub const fn ortho_pin_mask<const COLOUR: bool>(&self) -> u64
    where BoolExists<{!COLOUR}>: Sized {
        self.hor_pin_mask() | self.vert_pin_mask()
    }

    pub const fn lr_pin_mask<const COLOUR: bool>(&self) -> u64
    where BoolExists<{!COLOUR}>: Sized {
        let kings = self.col_king_mask::<COLOUR>();
        let pieces = self.piece_mask();
        //let own_pieces = self.col_piece_mask::<COLOUR>();
        let other_diag = self.col_diagonal_mask::<{!COLOUR}>();

        let mut mask = 0;
        let r1 = Self::sliding_mask::<{Shift::Left}>(kings, 7, pieces, Self::LEFT_SIDE);
        let r2 = Self::sliding_mask::<{Shift::Left}>(r1 & pieces, 7, pieces, Self::LEFT_SIDE);
        if r2 & other_diag != 0 {
            mask |= r1 | r2;
        }

        let r1 = Self::sliding_mask::<{Shift::Right}>(kings, 7, pieces, Self::RIGHT_SIDE);
        let r2 = Self::sliding_mask::<{Shift::Right}>(r1 & pieces, 7, pieces, Self::RIGHT_SIDE);
        if r2 & other_diag != 0 {
            mask |= r1 | r2;
        }

        mask
    }

    pub const fn rl_pin_mask<const COLOUR: bool>(&self) -> u64
    where BoolExists<{!COLOUR}>: Sized {
        let kings = self.col_king_mask::<COLOUR>();
        let pieces = self.piece_mask();
        //let own_pieces = self.col_piece_mask::<COLOUR>();
        let other_diag = self.col_diagonal_mask::<{!COLOUR}>();

        let mut mask = 0;
        let r1 = Self::sliding_mask::<{Shift::Right}>(kings, 9, pieces, Self::LEFT_SIDE);
        let r2 = Self::sliding_mask::<{Shift::Right}>(r1 & pieces, 9, pieces, Self::LEFT_SIDE);
        if r2 & other_diag != 0 {
            mask |= r1 | r2;
        }

        let r1 = Self::sliding_mask::<{Shift::Left}>(kings, 9, pieces, Self::RIGHT_SIDE);
        let r2 = Self::sliding_mask::<{Shift::Left}>(r1 & pieces, 9, pieces, Self::RIGHT_SIDE);
        if r2 & other_diag != 0 {
            mask |= r1 | r2;
        }

        mask
    }

    pub const fn diagonal_pin_mask<const COLOUR: bool>(&self) -> u64
    where BoolExists<{!COLOUR}>: Sized {
        self.lr_pin_mask() | self.rl_pin_mask()
    }

    #[inline(always)]
    pub fn gen_pawn_moves<const TURN: bool, const WQ: bool,
    const WK: bool, const BQ: bool, const BK: bool, Mov: OnMove>(&self, on_move: &mut Mov)
    where BoolExists<{!TURN}>: Sized {
        let base_mask = self.enemy_or_empty::<TURN>() & self.check_mask::<TURN>();
        let hor_pins = self.hor_pin_mask::<TURN>();
        let ortho_pins = self.ortho_pin_mask::<TURN>();
        let lr_pins = self.lr_pin_mask::<TURN>();
        let rl_pins = self.rl_pin_mask::<TURN>();
        let diagonal_pins = self.diagonal_pin_mask::<TURN>();
        let empty = !self.piece_mask();
        let enemy = self.col_piece_mask::<{!TURN}>();

        let base_pawns = self.col_pawn_mask::<TURN>() & base_mask;
        let lr_pawns = base_pawns & !rl_pins & !ortho_pins;
        let up_pawns = base_pawns & !diagonal_pins & !hor_pins;
        let rl_pawns = base_pawns & !lr_pins & !ortho_pins;
        if TURN {
            let up1 = (empty >> 8) & up_pawns;
            let up2 = (empty >> 16) & up1 & (0xff << 8);
            let lr = (enemy >> 7) & lr_pawns & !Self::LEFT_SIDE;
            let rl = (enemy >> 9) & rl_pawns & !Self::RIGHT_SIDE;
            if up1 != 0 {
                let from_idx = up1.trailing_zeros() as u8;
                on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx + 8);
            }
            if up2 != 0 {
                let from_idx = up2.trailing_zeros() as u8;
                on_move.on_pawn_push2::<TURN, WQ, WK, BQ, BK>(self, from_idx);
            }
            if lr != 0 {
                let from_idx = lr.trailing_zeros() as u8;
                on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx + 7);
            }
            if rl != 0 {
                let from_idx = rl.trailing_zeros() as u8;
                on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx + 9);
            }
        }
        else {
            let up1 = (empty << 8) & up_pawns;
            let up2 = (empty << 16) & up1 & (0xff << (8 * 6));
            let lr = (enemy << 7) & lr_pawns & !Self::RIGHT_SIDE;
            let rl = (enemy << 9) & rl_pawns & !Self::LEFT_SIDE;
            if up1 != 0 {
                let from_idx = up1.trailing_zeros() as u8;
                on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx - 8);
            }
            if up2 != 0 {
                let from_idx = up2.trailing_zeros() as u8;
                on_move.on_pawn_push2::<TURN, WQ, WK, BQ, BK>(self, from_idx);
            }
            if lr != 0 {
                let from_idx = lr.trailing_zeros() as u8;
                on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx - 7);
            }
            if rl != 0 {
                let from_idx = rl.trailing_zeros() as u8;
                on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx - 9);
            }
        }
    }

    #[inline(always)]
    pub fn gen_pawn_moves_with_ep<const TURN: bool, const WQ: bool,
    const WK: bool, const BQ: bool, const BK: bool, Mov: OnMove>(&self, on_move: &mut Mov, sq: u8)
    where BoolExists<{!TURN}>: Sized {
        let base_mask = self.enemy_or_empty::<TURN>() & self.check_mask::<TURN>();
        let hor_pins = self.hor_pin_mask::<TURN>();
        let ortho_pins = self.ortho_pin_mask::<TURN>();
        let lr_pins = self.lr_pin_mask::<TURN>();
        let rl_pins = self.rl_pin_mask::<TURN>();
        let diagonal_pins = self.diagonal_pin_mask::<TURN>();
        let empty = !self.piece_mask();
        let ep = 1 << sq;
        let enemy = self.col_piece_mask::<{!TURN}>() | ep;

        let base_pawns = self.col_pawn_mask::<TURN>() & base_mask;
        let lr_pawns = base_pawns & !rl_pins & !ortho_pins;
        let up_pawns = base_pawns & !diagonal_pins & !hor_pins;
        let rl_pawns = base_pawns & !lr_pins & !ortho_pins;
        if TURN {
            let up1 = (empty >> 8) & up_pawns;
            let up2 = (empty >> 16) & up1 & (0xff << 8);
            let lr = (enemy >> 7) & lr_pawns & !Self::LEFT_SIDE;
            let rl = (enemy >> 9) & rl_pawns & !Self::RIGHT_SIDE;
            if up1 != 0 {
                let from_idx = up1.trailing_zeros() as u8;
                on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx + 8);
            }
            if up2 != 0 {
                let from_idx = up2.trailing_zeros() as u8;
                on_move.on_pawn_push2::<TURN, WQ, WK, BQ, BK>(self, from_idx);
            }
            if lr != 0 {
                let from_idx = lr.trailing_zeros() as u8;
                if empty & !lr_pins & !rl_pins & !hor_pins & (from_idx + 7) as u64 != 0 {
                    on_move.on_ep_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx + 7);
                } else {
                    on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx + 7);
                }
            }
            if rl != 0 {
                let from_idx = rl.trailing_zeros() as u8;
                if empty & !lr_pins & !rl_pins & !hor_pins & (from_idx + 9) as u64 != 0 {
                    on_move.on_ep_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx + 9);
                } else {
                    on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx + 9);
                }
            }
        }
        else {
            let up1 = (empty << 8) & up_pawns;
            let up2 = (empty << 16) & up1 & (0xff << (8 * 6));
            let lr = (enemy << 7) & lr_pawns & !Self::RIGHT_SIDE;
            let rl = (enemy << 9) & rl_pawns & !Self::LEFT_SIDE;
            if up1 != 0 {
                let from_idx = up1.trailing_zeros() as u8;
                on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx - 8);
            }
            if up2 != 0 {
                let from_idx = up2.trailing_zeros() as u8;
                on_move.on_pawn_push2::<TURN, WQ, WK, BQ, BK>(self, from_idx);
            }
            if lr != 0 {
                let from_idx = lr.trailing_zeros() as u8;
                if empty & !lr_pins & !rl_pins & !hor_pins & (from_idx - 7) as u64 != 0 {
                    on_move.on_ep_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx - 7);
                } else {
                    on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx - 7);
                }
            }
            if rl != 0 {
                let from_idx = rl.trailing_zeros() as u8;
                if empty & !lr_pins & !rl_pins & !hor_pins & (from_idx - 9) as u64 != 0 {
                    on_move.on_ep_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx - 9);
                } else {
                    on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx - 9);
                }
            }
        }
    }

    #[inline(always)]
    pub fn gen_knight_moves<const TURN: bool, const WQ: bool,
    const WK: bool, const BQ: bool, const BK: bool, Mov: OnMove>(&self, on_move: &mut Mov)
    where BoolExists<{!TURN}>: Sized {
        let base_mask = self.enemy_or_empty::<TURN>() & self.check_mask::<TURN>();
        let ortho_pins = self.ortho_pin_mask::<TURN>();
        let diagonal_pins = self.diagonal_pin_mask::<TURN>();

        let mut knights = self.col_knight_mask::<TURN>() & !ortho_pins & !diagonal_pins;
        while knights != 0 {
            let from = knights & !(knights - 1);
            let from_idx = from.trailing_zeros() as u8;
            let mut to_mask = self.knight_like_attack_mask(from & self.col_knight_mask::<TURN>()) & base_mask;
            while to_mask != 0 {
                let to_idx = (to_mask & !(to_mask - 1)).trailing_zeros() as u8;
                on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, to_idx);
                to_mask &= to_mask - 1;
            }
            knights &= knights - 1;
        }
    }

    #[inline(always)]
    pub fn gen_diagonal_moves<const TURN: bool, const WQ: bool,
    const WK: bool, const BQ: bool, const BK: bool, Mov: OnMove>(&self, on_move: &mut Mov)
    where BoolExists<{!TURN}>: Sized {
        let base_mask = self.enemy_or_empty::<TURN>() & self.check_mask::<TURN>();
        let ortho_pins = self.ortho_pin_mask::<TURN>();
        let diagonal_pins = self.diagonal_pin_mask::<TURN>();

        let mut free_bishops = self.col_diagonal_mask::<TURN>() & !diagonal_pins & !ortho_pins;
        let mut pin_bishops = self.col_diagonal_mask::<TURN>() & diagonal_pins;

        while free_bishops != 0 {
            let from = free_bishops & !(free_bishops - 1);
            let from_idx = from.trailing_zeros() as u8;
            let mut to_mask = self.diagonal_like_attack_mask(from & self.col_diagonal_mask::<TURN>()) & base_mask;
            while to_mask != 0 {
                let to_idx = (to_mask & !(to_mask - 1)).trailing_zeros() as u8;
                on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, to_idx);
                to_mask &= to_mask - 1;
            }
            free_bishops &= free_bishops - 1;
        }

        while pin_bishops != 0 {
            let from = pin_bishops & !(pin_bishops - 1);
            let from_idx = from.trailing_zeros() as u8;
            let mut to_mask = self.diagonal_like_attack_mask(from & self.col_diagonal_mask::<TURN>()) & base_mask;
            while to_mask != 0 {
                let to_idx = (to_mask & !(to_mask - 1)).trailing_zeros() as u8;
                on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, to_idx);
                to_mask &= to_mask - 1;
            }
            pin_bishops &= pin_bishops - 1;
        }
    }

    #[inline(always)]
    pub fn gen_ortho_moves<const TURN: bool, const WQ: bool,
    const WK: bool, const BQ: bool, const BK: bool, Mov: OnMove>(&self, on_move: &mut Mov)
    where BoolExists<{!TURN}>: Sized {
        let base_mask = self.enemy_or_empty::<TURN>() & self.check_mask::<TURN>();
        let ortho_pins = self.ortho_pin_mask::<TURN>();
        let diagonal_pins = self.diagonal_pin_mask::<TURN>();

        let mut free_rooks = self.col_ortho_mask::<TURN>() & !diagonal_pins & !ortho_pins;
        let mut pin_rooks = self.col_ortho_mask::<TURN>() & diagonal_pins;

        while free_rooks != 0 {
            let from = free_rooks & !(free_rooks - 1);
            let from_idx = from.trailing_zeros() as u8;
            let mut to_mask = self.ortho_like_attack_mask(from & self.col_ortho_mask::<TURN>()) & base_mask;
            while to_mask != 0 {
                let to_idx = (to_mask & !(to_mask - 1)).trailing_zeros() as u8;
                on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, to_idx);
                to_mask &= to_mask - 1;
            }
            free_rooks &= free_rooks - 1;
        }

        while pin_rooks != 0 {
            let from = pin_rooks & !(pin_rooks - 1);
            let from_idx = from.trailing_zeros() as u8;
            let mut to_mask = self.ortho_like_attack_mask(from & self.col_ortho_mask::<TURN>()) & base_mask;
            while to_mask != 0 {
                let to_idx = (to_mask & !(to_mask - 1)).trailing_zeros() as u8;
                on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, to_idx);
                to_mask &= to_mask - 1;
            }
            pin_rooks &= pin_rooks - 1;
        }
    }

    #[inline(always)]
    pub fn gen_king_moves<const TURN: bool, const WQ: bool,
    const WK: bool, const BQ: bool, const BK: bool, Mov: OnMove>(&self, on_move: &mut Mov)
    where BoolExists<{!TURN}>: Sized {
        let other_attacks = self.attack_mask::<{!TURN}>();
        let base_mask = self.enemy_or_empty::<TURN>() & !other_attacks;
        let king = self.col_king_mask::<TURN>();

        let from_idx = king.trailing_zeros() as u8;  
        let mut to_mask = self.king_attack_mask::<TURN>() & base_mask;
        while to_mask != 0 {
            let to_idx = (to_mask & !(to_mask - 1)).trailing_zeros() as u8;
            on_move.on_king_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, to_idx);
            to_mask &= to_mask - 1;
        }
        if (WK || BK) && (king >> 1) & base_mask != 0 && (king >> 2) & base_mask != 0 {
            on_move.on_ks_castle::<TURN, WQ, WK, BQ, BK>(self);
        }
        if (WQ || BQ) && (king << 1) & base_mask != 0 && (king << 2) & base_mask != 0 {
            on_move.on_qs_castle::<TURN, WQ, WK, BQ, BK>(self);
        }
    }

    #[inline(always)]
    pub fn gen_moves<const TURN: bool, const WQ: bool,
    const WK: bool, const BQ: bool, const BK: bool, Mov: OnMove>(&self, on_move: &mut Mov)
    where BoolExists<{!TURN}>: Sized {
        self.gen_pawn_moves::<TURN, WQ, WK, BQ, BK, Mov>(on_move);
        self.gen_knight_moves::<TURN, WQ, WK, BQ, BK, Mov>(on_move);
        self.gen_diagonal_moves::<TURN, WQ, WK, BQ, BK, Mov>(on_move);
        self.gen_ortho_moves::<TURN, WQ, WK, BQ, BK, Mov>(on_move);
        self.gen_king_moves::<TURN, WQ, WK, BQ, BK, Mov>(on_move);
    }

    #[inline(always)]
    pub fn gen_moves_with_ep<const TURN: bool, const WQ: bool,
    const WK: bool, const BQ: bool, const BK: bool, Mov: OnMove>(&self, on_move: &mut Mov, ep: u8)
    where BoolExists<{!TURN}>: Sized {
        self.gen_pawn_moves_with_ep::<TURN, WQ, WK, BQ, BK, Mov>(on_move, ep);
        self.gen_knight_moves::<TURN, WQ, WK, BQ, BK, Mov>(on_move);
        self.gen_diagonal_moves::<TURN, WQ, WK, BQ, BK, Mov>(on_move);
        self.gen_ortho_moves::<TURN, WQ, WK, BQ, BK, Mov>(on_move);
        self.gen_king_moves::<TURN, WQ, WK, BQ, BK, Mov>(on_move);
    }
}

pub struct BitBoardGame {
    pub board: BitBoard,
    turn: PlayerColour,
}

impl ChessGame for BitBoardGame {
    type Move = BBMove;

    type UnMove = BitBoard;

    fn new() -> Self {
        todo!()
    }

    fn from_fen(fen: &str) -> Option<Self> {
        let mut fen_parts = fen.trim().split(" ");
        let fenboard = fen_parts.next()?;
        let turn = match fen_parts.next()? {
            "w" => PlayerColour::White,
            "b" => PlayerColour::Black,
            _ => return None
        };

        let castle_rights = fen_parts.next()?;
        let white_ks_castle = castle_rights.contains('K');
        let white_qs_castle = castle_rights.contains('Q');
        let black_ks_castle = castle_rights.contains('k');
        let black_qs_castle = castle_rights.contains('q');

        let enpassant_col = match fen_parts.next()?.chars().next()? {
            'a' => Some(0),
            'b' => Some(1),
            'c' => Some(2),
            'd' => Some(3),
            'e' => Some(4),
            'f' => Some(5),
            'g' => Some(6),
            'h' => Some(7),
            _ => None,
        };

        let mut board: [u64; 4] = [0; 4];
        let mut counter = 0;
        for c in fenboard.replace('/',"").chars() {
            if c.is_digit(10) {
                counter += c.to_digit(10)?;
                continue;
            }

            let mut piece_idx = match c.to_ascii_uppercase() {
                'P' => { 0b100 }
                'N' => { 0b101 }
                'B' => { 0b001 }
                'R' => {
                    if counter == 0  && black_qs_castle ||
                        counter == 7  && black_ks_castle ||
                        counter == 56 && white_qs_castle ||
                        counter == 63 && white_ks_castle {
                        0b110
                    }
                    else {
                        0b010
                    }
                }
                'Q' => { 0b011 }
                'K' => { 0b111 }
                _ => return None
            };
            piece_idx |= if c.is_ascii_uppercase() {0b1000} else {0};
            board.iter_mut().enumerate().for_each(|(i, v)| *v |= ((piece_idx >> i) & 1) << (63 - counter));
            counter += 1;
        }

        match enpassant_col {
            Some(x) => board[0] |= 1 << (if turn == PlayerColour::White {24} else {48} + x),
            None => {}
        }

        if counter == 64 {
            Some(BitBoardGame { board: BitBoard { board }, turn})
        } else {
            None
        }
    }

    fn decode_alg(&mut self, mov: &AlgebraicMove) -> Self::Move {
        todo!()
    }

    fn moves(&mut self) -> Vec<Self::Move> {
        let col_mask = match self.turn {
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

pub fn print_bitmask(mask: u64) {
    let mut bstr = String::from("");
    for i in 0..64 {
        if i % 8 == 0 {bstr.push('\n')}
        bstr.push(if (mask >> 63 - i) & 1 == 0 {'-'} else {'X'});
    }
    println!("{:b}", mask);
    println!("{}", bstr);
}

impl Display for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut bstr = String::from("");

        for i in 0..64 {
            let mask = 1 << 63 - i;
            let is_white = self.board[3] & mask != 0;

            let c = match (self.board[2] & mask != 0, self.board[1] & mask != 0, self.board[0] & mask != 0) {
                (false,false,false) => if is_white {'*'} else {'-'},
                (true ,false,false) => 'p',
                (true ,false,true ) => 'n',
                (false,false,true ) => 'b',
                (_    ,true ,false) => 'r',
                (false,true ,true ) => 'q',
                (true ,true ,true ) => 'k'
            };

            bstr.push(if is_white {c.to_ascii_uppercase()} else {c});
            if i % 8 == 7 {
                bstr.push('\n');
            }
        }
        f.write_fmt(format_args!("{}", bstr))
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