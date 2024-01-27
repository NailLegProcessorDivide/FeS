use std::{fmt::Display, marker::ConstParamTy};

use crate::{notation::AlgebraicMove, game::{ChessGame, Move}};

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
    _packed: u16
}

pub struct BoolExists<const _N: bool>{}

/// Column-wise representation of chess board (if you stack each u64 on top of each other)
/// 0000 => none
/// 1000 => --unused--
/// 1??? => white
/// 0??? => black
/// ?001 => bishop
/// ?010 => rook
/// ?011 => queen
/// ?100 => pawn
/// ?101 => knight
/// ?110 => --unused--
/// ?111 => king
#[derive(Clone)]
pub struct BitBoard {
    // Index that corresponds to each bit: 0b3210
    board: [u64; 4]
}

#[derive(Clone)]
pub struct BitBoardGameMove {
    mov: u16,
    bbg: BitBoardGame 
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
            *line |= tmp << to;
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
    pub const fn colour_mask<const TURN: bool>(&self) -> u64 {
        if TURN {
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
    pub const fn col_piece_mask<const TURN: bool>(&self) -> u64 {
        self.piece_mask() & self.colour_mask::<TURN>()
    }

    #[inline(always)]
    pub const fn enemy_or_empty<const TURN: bool>(&self) -> u64 {
        !self.col_piece_mask::<TURN>()
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
    pub const fn col_pawn_mask<const TURN: bool>(&self) -> u64 {
        self.pawn_mask() & self.colour_mask::<TURN>()
    }

    /// colour 0 = white, u64::MAX = black
    /// 1 if colour pawn
    /// 0 if no colour pawn
    #[inline(always)]
    pub const fn pawn_attack_mask<const TURN: bool>(&self) -> u64 {
        let pawns = self.col_pawn_mask::<TURN>();
        if TURN {
            ((pawns << 9) & !Self::RIGHT_SIDE) |
            ((pawns << 7) & !Self::LEFT_SIDE)
        }
        else {
            ((pawns >> 7) & !Self::RIGHT_SIDE) |
            ((pawns >> 9) & !Self::LEFT_SIDE)
        }
    }

    #[inline(always)]
    pub const fn pawn_like_attack_mask<const TURN: bool>(&self, pieces: u64) -> u64 {
        if TURN {
            ((pieces << 9) & !Self::RIGHT_SIDE) |
            ((pieces << 7) & !Self::LEFT_SIDE)
        }
        else {
            ((pieces >> 7) & !Self::RIGHT_SIDE) |
            ((pieces >> 9) & !Self::LEFT_SIDE)
        }
    }

    #[inline(always)]
    pub const fn pawn_move_mask<const TURN: bool>(&self) -> u64 {
        let pawns = self.col_pawn_mask::<TURN>();
        let pieces = self.piece_mask();

        if TURN {
            let step = (pawns << 8) & !pieces;
            step | ((step << 8) & !pieces & 0xff000000)
        } else {
            let step = (pawns >> 8) & !pieces;
            step | ((step >> 8) & !pieces & 0xff00000000)
        }
    }

    #[inline(always)]
    pub const fn pawn_like_move_mask<const TURN: bool>(&self, pieces: u64) -> u64 {
        let blockers = self.piece_mask();

        if TURN {
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
    pub const fn col_knight_mask<const TURN: bool>(&self) -> u64 {
        self.knight_mask() & (self.colour_mask::<TURN>())
    }

    /// colour 0 = white, u64::MAX = black
    /// 1 if colour knight can attack
    /// 0 if no colour knight cant attack
    #[inline(always)]
    pub const fn knight_attack_mask<const TURN: bool>(&self) -> u64 {
        let knights = self.col_knight_mask::<TURN>();
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
    pub const fn col_diagonal_mask<const TURN: bool>(&self) -> u64 {
        self.diagonal_mask() & (self.colour_mask::<TURN>())
    }

    /// colour 0 = white, u64::MAX = black
    /// 1 if colour bishop can attack
    /// 0 if no colour bishop cant attack
    /// Note: a queen is a bishop
    #[inline(always)]
    pub const fn diagonal_attack_mask<const TURN: bool>(&self) -> u64 {
        let bishops = self.col_diagonal_mask::<TURN>();
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
        self.board[1] & !self.board[2]
    }

    /// colour 0 = white, u64::MAX = black
    /// 1 if colour rook like
    /// 0 if no colour rook like
    #[inline(always)]
    pub const fn col_ortho_mask<const TURN: bool>(&self) -> u64 {
        self.ortho_mask() & (self.colour_mask::<TURN>())
    }

    /// colour 0 = white, u64::MAX = black
    /// 1 if colour rook can attack
    /// 0 if no colour rook cant attack
    /// Note: a queen is a rook
    #[inline(always)]
    pub const fn ortho_attack_mask<const TURN: bool>(&self) -> u64 {
        let rooks = self.col_ortho_mask::<TURN>();
        self.ortho_like_attack_mask(rooks)
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
    pub const fn col_king_mask<const TURN: bool>(&self) -> u64 {
        self.king_mask() & (self.colour_mask::<TURN>())
    }

    /// colour 0 = white, u64::MAX = black
    /// 1 if colour king can attack
    /// 0 if no colour king cant attack
    #[inline(always)]
    pub const fn king_attack_mask<const TURN: bool>(&self) -> u64 {
        let kings = self.col_king_mask::<TURN>();
        let u = kings << 8;
        let d = kings >> 8;
        let mast =  kings | u | d;
        ((mast >> 1) & !Self::LEFT_SIDE) | ((mast << 1) & !Self::RIGHT_SIDE) | u | d
    }

    #[inline(always)]
    pub const fn attack_mask<const TURN: bool>(&self) -> u64 {
        self.pawn_attack_mask::<TURN>() |
            self.knight_attack_mask::<TURN>() |
            self.diagonal_attack_mask::<TURN>() |
            self.ortho_attack_mask::<TURN>() |
            self.king_attack_mask::<TURN>()
    }

    #[inline(always)]
    pub const fn hor_check_mask<const TURN: bool>(&self) -> u64
    where BoolExists<{!TURN}>: Sized {
        let kings = self.col_king_mask::<TURN>();
        let pieces = self.piece_mask();
        let other_ortho = self.col_ortho_mask::<{!TURN}>();

        let mut mask = u64::MAX;

        let r1 = Self::sliding_mask::<{Shift::Right}>(kings, 1, pieces, Self::LEFT_SIDE);
        if r1 & other_ortho != 0 {
            mask &= r1;
        }

        let r1 = Self::sliding_mask::<{Shift::Left}>(kings, 1, pieces, Self::RIGHT_SIDE);
        if r1 & other_ortho != 0 {
            mask &= r1;
        }

        mask
    }

    #[inline(always)]
    pub const fn vert_check_mask<const TURN: bool>(&self) -> u64
    where BoolExists<{!TURN}>: Sized {
        let kings = self.col_king_mask::<TURN>();
        let pieces = self.piece_mask();
        let other_ortho = self.col_ortho_mask::<{!TURN}>();

        let mut mask = u64::MAX;

        let r1 = Self::sliding_mask::<{Shift::Left}>(kings, 8, pieces, 0);
        if r1 & other_ortho != 0 {
            mask &= r1;
        }

        let r1 = Self::sliding_mask::<{Shift::Right}>(kings, 8, pieces, 0);
        if r1 & other_ortho != 0 {
            mask &= r1;
        }

        mask
    }

    #[inline(always)]
    pub const fn lr_check_mask<const TURN: bool>(&self) -> u64
    where BoolExists<{!TURN}>: Sized {
        let kings = self.col_king_mask::<TURN>();
        let pieces = self.piece_mask();
        let other_diag = self.col_diagonal_mask::<{!TURN}>();
        
        let mut mask = u64::MAX;

        let r1 = Self::sliding_mask::<{Shift::Left}>(kings, 7, pieces, Self::LEFT_SIDE);
        if r1 & other_diag != 0 {
            mask &= r1;
        }

        let r1 = Self::sliding_mask::<{Shift::Right}>(kings, 7, pieces, Self::RIGHT_SIDE);
        if r1 & other_diag != 0 {
            mask &= r1;
        }

        mask
    }

    #[inline(always)]
    pub const fn rl_check_mask<const TURN: bool>(&self) -> u64
    where BoolExists<{!TURN}>: Sized {
        let kings = self.col_king_mask::<TURN>();
        let pieces = self.piece_mask();
        let other_diag = self.col_diagonal_mask::<{!TURN}>();
        
        let mut mask = u64::MAX;

        let r1 = Self::sliding_mask::<{Shift::Right}>(kings, 9, pieces, Self::LEFT_SIDE);
        if r1 & other_diag != 0 {
            mask &= r1;
        }

        let r1 = Self::sliding_mask::<{Shift::Left}>(kings, 9, pieces, Self::RIGHT_SIDE);
        if r1 & other_diag != 0 {
            mask &= r1;
        }

        mask
    }

    #[inline(always)]
    pub const fn check_mask<const TURN: bool>(&self) -> u64
    where BoolExists<{!TURN}>: Sized {
        let mut mask = u64::MAX;

        mask &= self.hor_check_mask::<TURN>() & self.vert_check_mask::<TURN>() &
                    self.lr_check_mask::<TURN>() & self.rl_check_mask::<TURN>();
      
        let kings = self.col_king_mask::<TURN>();

        let other_knights = Self::col_knight_mask::<{!TURN}>(self);
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

        let other_pawns = Self::col_pawn_mask::<{!TURN}>(self);

        if TURN {
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
    pub const fn vert_pin_mask<const TURN: bool>(&self) -> u64
    where BoolExists<{!TURN}>: Sized {
        let kings = self.col_king_mask::<TURN>();
        let pieces = self.piece_mask();
        //let own_pieces = self.col_piece_mask::<TURN>();
        let other_ortho = self.col_ortho_mask::<{!TURN}>();
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
    pub const fn hor_pin_mask<const TURN: bool>(&self) -> u64
    where BoolExists<{!TURN}>: Sized {
        let kings = self.col_king_mask::<TURN>();
        let pieces = self.piece_mask();
        //let own_pieces = self.col_piece_mask::<TURN>();
        let other_ortho = self.col_ortho_mask::<{!TURN}>();
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
    pub const fn ortho_pin_mask<const TURN: bool>(&self) -> u64
    where BoolExists<{!TURN}>: Sized {
        self.hor_pin_mask() | self.vert_pin_mask()
    }

    pub const fn lr_pin_mask<const TURN: bool>(&self) -> u64
    where BoolExists<{!TURN}>: Sized {
        let kings = self.col_king_mask::<TURN>();
        let pieces = self.piece_mask();
        //let own_pieces = self.col_piece_mask::<TURN>();
        let other_diag = self.col_diagonal_mask::<{!TURN}>();

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

    pub const fn rl_pin_mask<const TURN: bool>(&self) -> u64
    where BoolExists<{!TURN}>: Sized {
        let kings = self.col_king_mask::<TURN>();
        let pieces = self.piece_mask();
        //let own_pieces = self.col_piece_mask::<TURN>();
        let other_diag = self.col_diagonal_mask::<{!TURN}>();

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

    pub const fn diagonal_pin_mask<const TURN: bool>(&self) -> u64
    where BoolExists<{!TURN}>: Sized {
        self.lr_pin_mask() | self.rl_pin_mask()
    }

    #[inline(always)]
    pub fn gen_pawn_moves<const TURN: bool, const WQ: bool,
    const WK: bool, const BQ: bool, const BK: bool, Mov: OnMove>(&self, on_move: &mut Mov)
    where BoolExists<{!TURN}>: Sized {
        let check_mask = self.check_mask::<TURN>();
        let hor_pins = self.hor_pin_mask::<TURN>();
        let ortho_pins = self.ortho_pin_mask::<TURN>();
        let lr_pins = self.lr_pin_mask::<TURN>();
        let rl_pins = self.rl_pin_mask::<TURN>();
        let diagonal_pins = self.diagonal_pin_mask::<TURN>();
        let empty = !self.piece_mask();
        let empty_free = empty & check_mask;
        let enemy = self.col_piece_mask::<{!TURN}>() & check_mask;

        let base_pawns = self.col_pawn_mask::<TURN>();
        let lr_pawns = base_pawns & !rl_pins & !ortho_pins;
        let up_pawns = base_pawns & !diagonal_pins & !hor_pins;
        let rl_pawns = base_pawns & !lr_pins & !ortho_pins;
        if TURN {
            let mut up1 = (empty_free >> 8) & up_pawns;
            let mut up2 = (empty_free >> 16) & (empty >> 8) & up_pawns & (0xff << 8);
            let mut lr = (enemy >> 7) & lr_pawns & !Self::RIGHT_SIDE;
            let mut rl = (enemy >> 9) & rl_pawns & !Self::LEFT_SIDE;
            while up1 != 0 {
                let from_idx = up1.trailing_zeros() as u8;
                on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx + 8);
                up1 &= up1 - 1;
            }
            while up2 != 0 {
                let from_idx = up2.trailing_zeros() as u8;
                on_move.on_pawn_push2::<TURN, WQ, WK, BQ, BK>(self, from_idx);
                up2 &= up2 - 1;
            }
            while lr != 0 {
                let from_idx = lr.trailing_zeros() as u8;
                on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx + 7);
                lr &= lr - 1;
            }
            while rl != 0 {
                let from_idx = rl.trailing_zeros() as u8;
                on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx + 9);
                rl &= rl - 1;
            }
        }
        else {
            let mut up1 = (empty_free << 8) & up_pawns;
            let mut up2 = (empty_free << 16) & (empty << 8) & up_pawns & (0xff << (8 * 6));
            let mut lr = (enemy << 7) & lr_pawns & !Self::LEFT_SIDE;
            let mut rl = (enemy << 9) & rl_pawns & !Self::RIGHT_SIDE;
            while up1 != 0 {
                let from_idx = up1.trailing_zeros() as u8;
                on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx - 8);
                up1 &= up1 - 1;
            }
            while up2 != 0 {
                let from_idx = up2.trailing_zeros() as u8;
                on_move.on_pawn_push2::<TURN, WQ, WK, BQ, BK>(self, from_idx);
                up2 &= up2 - 1;
            }
            while lr != 0 {
                let from_idx = lr.trailing_zeros() as u8;
                on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx - 7);
                lr &= lr - 1;
            }
            while rl != 0 {
                let from_idx = rl.trailing_zeros() as u8;
                on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx - 9);
                rl &= rl - 1;
            }
        }
    }

    #[inline(always)]
    pub fn gen_pawn_moves_with_ep<const TURN: bool, const WQ: bool,
    const WK: bool, const BQ: bool, const BK: bool, Mov: OnMove>(&self, on_move: &mut Mov, sq: u8)
    where BoolExists<{!TURN}>: Sized {
        let check_mask = self.check_mask::<TURN>();
        let hor_pins = self.hor_pin_mask::<TURN>();
        let ortho_pins = self.ortho_pin_mask::<TURN>();
        let lr_pins = self.lr_pin_mask::<TURN>();
        let rl_pins = self.rl_pin_mask::<TURN>();
        let diagonal_pins = self.diagonal_pin_mask::<TURN>();
        let empty = !self.piece_mask();
        let empty_free = empty & check_mask;
        let ep = 1 << sq;
        let enemy = self.col_piece_mask::<{!TURN}>() | ep;

        let base_pawns = self.col_pawn_mask::<TURN>();
        let lr_pawns = base_pawns & !rl_pins & !ortho_pins;
        let up_pawns = base_pawns & !diagonal_pins & !hor_pins;
        let rl_pawns = base_pawns & !lr_pins & !ortho_pins;
        if TURN {
            let mut up1 = (empty_free >> 8) & up_pawns;
            let mut up2 = (empty_free >> 16) & (empty >> 8) & up_pawns & (0xff << 8);
            let mut lr = (enemy >> 7) & lr_pawns & !Self::RIGHT_SIDE;
            let mut rl = (enemy >> 9) & rl_pawns & !Self::LEFT_SIDE;
            while up1 != 0 {
                let from_idx = up1.trailing_zeros() as u8;
                on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx + 8);
                up1 &= up1 - 1;
            }
            while up2 != 0 {
                let from_idx = up2.trailing_zeros() as u8;
                on_move.on_pawn_push2::<TURN, WQ, WK, BQ, BK>(self, from_idx);
                up2 &= up2 - 1;
            }
            while lr != 0 {
                let from_idx = lr.trailing_zeros() as u8;
                if empty & (1 << (from_idx + 7)) != 0 {
                    on_move.on_ep_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx + 7);
                } else {
                    on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx + 7);
                }
                lr &= lr - 1;
            }
            while rl != 0 {
                let from_idx = rl.trailing_zeros() as u8;
                if empty & (1 << (from_idx + 9)) != 0 {
                    on_move.on_ep_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx + 9);
                } else {
                    on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx + 9);
                }
                rl &= rl - 1;
            }
        }
        else {
            let mut up1 = (empty_free << 8) & up_pawns;
            let mut up2 = (empty_free << 16) & (empty << 8) & up_pawns & (0xff << (8 * 6));
            let mut lr = (enemy << 7) & lr_pawns & !Self::LEFT_SIDE;
            let mut rl = (enemy << 9) & rl_pawns & !Self::RIGHT_SIDE;
            while up1 != 0 {
                let from_idx = up1.trailing_zeros() as u8;
                on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx - 8);
                up1 &= up1 - 1;
            }
            while up2 != 0 {
                let from_idx = up2.trailing_zeros() as u8;
                on_move.on_pawn_push2::<TURN, WQ, WK, BQ, BK>(self, from_idx);
                up2 &= up2 - 1;
            }
            while lr != 0 {
                let from_idx = lr.trailing_zeros() as u8;
                if empty & (1 << (from_idx - 7)) != 0 {
                    on_move.on_ep_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx - 7);
                } else {
                    on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx - 7);
                }
                lr &= lr - 1;
            }
            while rl != 0 {
                let from_idx = rl.trailing_zeros() as u8;
                if empty & (1 << (from_idx - 9)) != 0 {
                    on_move.on_ep_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx - 9);
                } else {
                    on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, from_idx - 9);
                }
                rl &= rl - 1;
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
            let from_idx = knights.trailing_zeros() as u8;
            let mut to_mask = self.knight_like_attack_mask(1 << from_idx) & base_mask;
            while to_mask != 0 {
                let to_idx = to_mask.trailing_zeros() as u8;
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
            let from_idx = free_bishops.trailing_zeros() as u8;
            let mut to_mask = self.diagonal_like_attack_mask(1 << from_idx) & base_mask;
            while to_mask != 0 {
                let to_idx = to_mask.trailing_zeros() as u8;
                on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, to_idx);
                to_mask &= to_mask - 1;
            }
            free_bishops &= free_bishops - 1;
        }

        while pin_bishops != 0 {
            let from_idx = pin_bishops.trailing_zeros() as u8;
            let mut to_mask = self.diagonal_like_attack_mask(1 << from_idx) & base_mask & diagonal_pins;
            while to_mask != 0 {
                let to_idx = to_mask.trailing_zeros() as u8;
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
        let mut pin_rooks = self.col_ortho_mask::<TURN>() & ortho_pins;

        while free_rooks != 0 {
            let from_idx = free_rooks.trailing_zeros() as u8;
            let mut to_mask = self.ortho_like_attack_mask(1 << from_idx) & base_mask;
            while to_mask != 0 {
                let to_idx = to_mask.trailing_zeros() as u8;
                on_move.on_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, to_idx);
                to_mask &= to_mask - 1;
            }
            free_rooks &= free_rooks - 1;
        }

        while pin_rooks != 0 {
            let from_idx = pin_rooks.trailing_zeros() as u8;
            let mut to_mask = self.ortho_like_attack_mask(1 << from_idx) & base_mask & ortho_pins;
            while to_mask != 0 {
                let to_idx = to_mask.trailing_zeros() as u8;
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
        let empty = !self.piece_mask();
        let enemies = self.col_piece_mask::<{!TURN}>();
        let other_attacks = self.attack_mask::<{!TURN}>();
        let base_mask = self.enemy_or_empty::<TURN>() & !other_attacks;
        let king = self.col_king_mask::<TURN>();

        let from_idx = king.trailing_zeros() as u8;
        let mut to_mask = self.king_attack_mask::<TURN>() & base_mask;
        
        if self.hor_check_mask::<TURN>() != u64::MAX {
            to_mask &= !((!Self::LEFT_SIDE & (king >> 1) | !Self::RIGHT_SIDE & (king << 1)) & !enemies);
        } 

        if self.vert_check_mask::<TURN>() != u64::MAX {
            to_mask &= !(((king >> 8) | (king << 8)) & !enemies);
        }

        if self.lr_check_mask::<TURN>() != u64::MAX {
            to_mask &= !((!Self::RIGHT_SIDE & (king >> 7) | !Self::LEFT_SIDE & (king << 7)) & !enemies);

        } 

        if self.rl_check_mask::<TURN>() != u64::MAX {
            to_mask &= !((!Self::LEFT_SIDE & (king >> 9) | !Self::RIGHT_SIDE & (king << 9)) & !enemies);
        }

        while to_mask != 0 {
            let to_idx = to_mask.trailing_zeros() as u8;
            on_move.on_king_move::<TURN, WQ, WK, BQ, BK>(self, from_idx, to_idx);
            to_mask &= to_mask - 1;
        }

        if WK && ((0b00000110 & empty) + 8) & !other_attacks == 0b00001110 {
            on_move.on_ks_castle::<TURN, WQ, WK, BQ, BK>(self);
        }

        if BK && ((0b00000110 & (empty >> 56)) + 8) & (!other_attacks >> 56) == 0b00001110 {
            on_move.on_ks_castle::<TURN, WQ, WK, BQ, BK>(self);
        }

        if WQ && ((0b01110000 & empty) >> 1) & !other_attacks == 0b00111000 {
            on_move.on_qs_castle::<TURN, WQ, WK, BQ, BK>(self);
        }

        if BQ && ((0b01110000 & (empty >> 56)) >> 1) & (!other_attacks >> 56) == 0b00111000 {
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

#[derive(Clone)]
pub struct BitBoardGame {
    pub board: BitBoard,
    turn: bool,
    white_qs: bool,
    white_ks: bool,
    black_qs: bool,
    black_ks: bool,
    ep: Option<u8>,
}

impl ChessGame for BitBoardGame {
    type Move = BitBoardGameMove;

    type UnMove = BitBoardGame;

    fn new() -> Self {
        todo!()
    }

    fn from_fen(fen: &str) -> Option<Self> {
        let mut fen_parts = fen.trim().split(" ");
        let fenboard = fen_parts.next()?;
        let turn = match fen_parts.next()? {
            "w" => true,
            "b" => false,
            _ => return None
        };

        let castle_rights = fen_parts.next()?;
        let white_ks_castle = castle_rights.contains('K');
        let white_qs_castle = castle_rights.contains('Q');
        let black_ks_castle = castle_rights.contains('k');
        let black_qs_castle = castle_rights.contains('q');

        let enpassant_col = match fen_parts.next()?.chars().next()? {
            'a' => Some(7),
            'b' => Some(6),
            'c' => Some(5),
            'd' => Some(4),
            'e' => Some(3),
            'f' => Some(2),
            'g' => Some(1),
            'h' => Some(0),
            _ => None,
        };

        let enpassant = match enpassant_col {
            Some(x) => Some(if turn {x + 40} else {x + 16}),
            _ => None
        };

        let mut board: [u64; 4] = [0; 4];
        let mut counter = 0;
        for c in fenboard.replace('/',"").chars() {
            if c.is_digit(10) {
                counter += c.to_digit(10)?;
                continue;
            }

            let mut piece_idx = match c.to_ascii_uppercase() {
                'P' => 0b100,
                'N' => 0b101,
                'B' => 0b001,
                'R' => 0b010,
                'Q' => 0b011,
                'K' => 0b111,
                _ => return None
            };
            piece_idx |= if c.is_ascii_uppercase() {0b1000} else {0};
            board.iter_mut().enumerate().for_each(|(i, v)| *v |= ((piece_idx >> i) & 1) << (63 - counter));
            counter += 1;
        }

        if counter == 64 {
            Some(BitBoardGame { board: BitBoard { board }, turn, white_qs: white_qs_castle,
                     white_ks: white_ks_castle, black_qs: black_qs_castle, black_ks: black_ks_castle, ep: enpassant })
        } else {
            None
        }
    }

    fn decode_alg(&mut self, _mov: &AlgebraicMove) -> Self::Move {
        todo!()
    }

    fn moves(&mut self) -> Vec<Self::Move> {
        let mut genny = GenericMoveGenerator { next: Vec::new()};
        match (self.turn, self.white_qs, self.white_ks, self.black_qs, self.black_ks, self.ep) {
            (true , true , true , true , true , None)     => self.board.gen_moves::<true , true , true , true , true , GenericMoveGenerator>(&mut genny),
            (true , true , true , true , true , Some(ep)) => self.board.gen_moves_with_ep::<true , true , true , true , true , GenericMoveGenerator>(&mut genny, ep),
            (true , true , true , true , false, None)     => self.board.gen_moves::<true , true , true , true , false, GenericMoveGenerator>(&mut genny),
            (true , true , true , true , false, Some(ep)) => self.board.gen_moves_with_ep::<true , true , true , true , false, GenericMoveGenerator>(&mut genny, ep),
            (true , true , true , false, true , None)     => self.board.gen_moves::<true , true , true , false, true , GenericMoveGenerator>(&mut genny),
            (true , true , true , false, true , Some(ep)) => self.board.gen_moves_with_ep::<true , true , true , false, true , GenericMoveGenerator>(&mut genny, ep),
            (true , true , true , false, false, None)     => self.board.gen_moves::<true , true , true , false, false, GenericMoveGenerator>(&mut genny),
            (true , true , true , false, false, Some(ep)) => self.board.gen_moves_with_ep::<true , true , true , false, false, GenericMoveGenerator>(&mut genny, ep),
            (true , true , false, true , true , None)     => self.board.gen_moves::<true , true , false, true , true , GenericMoveGenerator>(&mut genny),
            (true , true , false, true , true , Some(ep)) => self.board.gen_moves_with_ep::<true , true , false, true , true , GenericMoveGenerator>(&mut genny, ep),
            (true , true , false, true , false, None)     => self.board.gen_moves::<true , true , false, true , false, GenericMoveGenerator>(&mut genny),
            (true , true , false, true , false, Some(ep)) => self.board.gen_moves_with_ep::<true , true , false, true , false, GenericMoveGenerator>(&mut genny, ep),
            (true , true , false, false, true , None)     => self.board.gen_moves::<true , true , false, false, true , GenericMoveGenerator>(&mut genny),
            (true , true , false, false, true , Some(ep)) => self.board.gen_moves_with_ep::<true , true , false, false, true , GenericMoveGenerator>(&mut genny, ep),
            (true , true , false, false, false, None)     => self.board.gen_moves::<true , true , false, false, false, GenericMoveGenerator>(&mut genny),
            (true , true , false, false, false, Some(ep)) => self.board.gen_moves_with_ep::<true , true , false, false, false, GenericMoveGenerator>(&mut genny, ep),
            (true , false, true , true , true , None)     => self.board.gen_moves::<true , false, true , true , true , GenericMoveGenerator>(&mut genny),
            (true , false, true , true , true , Some(ep)) => self.board.gen_moves_with_ep::<true , false, true , true , true , GenericMoveGenerator>(&mut genny, ep),
            (true , false, true , true , false, None)     => self.board.gen_moves::<true , false, true , true , false, GenericMoveGenerator>(&mut genny),
            (true , false, true , true , false, Some(ep)) => self.board.gen_moves_with_ep::<true , false, true , true , false, GenericMoveGenerator>(&mut genny, ep),
            (true , false, true , false, true , None)     => self.board.gen_moves::<true , false, true , false, true , GenericMoveGenerator>(&mut genny),
            (true , false, true , false, true , Some(ep)) => self.board.gen_moves_with_ep::<true , false, true , false, true , GenericMoveGenerator>(&mut genny, ep),
            (true , false, true , false, false, None)     => self.board.gen_moves::<true , false, true , false, false, GenericMoveGenerator>(&mut genny),
            (true , false, true , false, false, Some(ep)) => self.board.gen_moves_with_ep::<true , false, true , false, false, GenericMoveGenerator>(&mut genny, ep),
            (true , false, false, true , true , None)     => self.board.gen_moves::<true , false, false, true , true , GenericMoveGenerator>(&mut genny),
            (true , false, false, true , true , Some(ep)) => self.board.gen_moves_with_ep::<true , false, false, true , true , GenericMoveGenerator>(&mut genny, ep),
            (true , false, false, true , false, None)     => self.board.gen_moves::<true , false, false, true , false, GenericMoveGenerator>(&mut genny),
            (true , false, false, true , false, Some(ep)) => self.board.gen_moves_with_ep::<true , false, false, true , false, GenericMoveGenerator>(&mut genny, ep),
            (true , false, false, false, true , None)     => self.board.gen_moves::<true , false, false, false, true , GenericMoveGenerator>(&mut genny),
            (true , false, false, false, true , Some(ep)) => self.board.gen_moves_with_ep::<true , false, false, false, true , GenericMoveGenerator>(&mut genny, ep),
            (true , false, false, false, false, None)     => self.board.gen_moves::<true , false, false, false, false, GenericMoveGenerator>(&mut genny),
            (true , false, false, false, false, Some(ep)) => self.board.gen_moves_with_ep::<true , false, false, false, false, GenericMoveGenerator>(&mut genny, ep),
            (false, true , true , true , true , None)     => self.board.gen_moves::<false, true , true , true , true , GenericMoveGenerator>(&mut genny),
            (false, true , true , true , true , Some(ep)) => self.board.gen_moves_with_ep::<false, true , true , true , true , GenericMoveGenerator>(&mut genny, ep),
            (false, true , true , true , false, None)     => self.board.gen_moves::<false, true , true , true , false, GenericMoveGenerator>(&mut genny),
            (false, true , true , true , false, Some(ep)) => self.board.gen_moves_with_ep::<false, true , true , true , false, GenericMoveGenerator>(&mut genny, ep),
            (false, true , true , false, true , None)     => self.board.gen_moves::<false, true , true , false, true , GenericMoveGenerator>(&mut genny),
            (false, true , true , false, true , Some(ep)) => self.board.gen_moves_with_ep::<false, true , true , false, true , GenericMoveGenerator>(&mut genny, ep),
            (false, true , true , false, false, None)     => self.board.gen_moves::<false, true , true , false, false, GenericMoveGenerator>(&mut genny),
            (false, true , true , false, false, Some(ep)) => self.board.gen_moves_with_ep::<false, true , true , false, false, GenericMoveGenerator>(&mut genny, ep),
            (false, true , false, true , true , None)     => self.board.gen_moves::<false, true , false, true , true , GenericMoveGenerator>(&mut genny),
            (false, true , false, true , true , Some(ep)) => self.board.gen_moves_with_ep::<false, true , false, true , true , GenericMoveGenerator>(&mut genny, ep),
            (false, true , false, true , false, None)     => self.board.gen_moves::<false, true , false, true , false, GenericMoveGenerator>(&mut genny),
            (false, true , false, true , false, Some(ep)) => self.board.gen_moves_with_ep::<false, true , false, true , false, GenericMoveGenerator>(&mut genny, ep),
            (false, true , false, false, true , None)     => self.board.gen_moves::<false, true , false, false, true , GenericMoveGenerator>(&mut genny),
            (false, true , false, false, true , Some(ep)) => self.board.gen_moves_with_ep::<false, true , false, false, true , GenericMoveGenerator>(&mut genny, ep),
            (false, true , false, false, false, None)     => self.board.gen_moves::<false, true , false, false, false, GenericMoveGenerator>(&mut genny),
            (false, true , false, false, false, Some(ep)) => self.board.gen_moves_with_ep::<false, true , false, false, false, GenericMoveGenerator>(&mut genny, ep),
            (false, false, true , true , true , None)     => self.board.gen_moves::<false, false, true , true , true , GenericMoveGenerator>(&mut genny),
            (false, false, true , true , true , Some(ep)) => self.board.gen_moves_with_ep::<false, false, true , true , true , GenericMoveGenerator>(&mut genny, ep),
            (false, false, true , true , false, None)     => self.board.gen_moves::<false, false, true , true , false, GenericMoveGenerator>(&mut genny),
            (false, false, true , true , false, Some(ep)) => self.board.gen_moves_with_ep::<false, false, true , true , false, GenericMoveGenerator>(&mut genny, ep),
            (false, false, true , false, true , None)     => self.board.gen_moves::<false, false, true , false, true , GenericMoveGenerator>(&mut genny),
            (false, false, true , false, true , Some(ep)) => self.board.gen_moves_with_ep::<false, false, true , false, true , GenericMoveGenerator>(&mut genny, ep),
            (false, false, true , false, false, None)     => self.board.gen_moves::<false, false, true , false, false, GenericMoveGenerator>(&mut genny),
            (false, false, true , false, false, Some(ep)) => self.board.gen_moves_with_ep::<false, false, true , false, false, GenericMoveGenerator>(&mut genny, ep),
            (false, false, false, true , true , None)     => self.board.gen_moves::<false, false, false, true , true , GenericMoveGenerator>(&mut genny),
            (false, false, false, true , true , Some(ep)) => self.board.gen_moves_with_ep::<false, false, false, true , true , GenericMoveGenerator>(&mut genny, ep),
            (false, false, false, true , false, None)     => self.board.gen_moves::<false, false, false, true , false, GenericMoveGenerator>(&mut genny),
            (false, false, false, true , false, Some(ep)) => self.board.gen_moves_with_ep::<false, false, false, true , false, GenericMoveGenerator>(&mut genny, ep),
            (false, false, false, false, true , None)     => self.board.gen_moves::<false, false, false, false, true , GenericMoveGenerator>(&mut genny),
            (false, false, false, false, true , Some(ep)) => self.board.gen_moves_with_ep::<false, false, false, false, true , GenericMoveGenerator>(&mut genny, ep),
            (false, false, false, false, false, None)     => self.board.gen_moves::<false, false, false, false, false, GenericMoveGenerator>(&mut genny),
            (false, false, false, false, false, Some(ep)) => self.board.gen_moves_with_ep::<false, false, false, false, false, GenericMoveGenerator>(&mut genny, ep),
        }
        genny.next
    }

    fn do_move(&mut self, mov: &Self::Move) -> Self::UnMove {
        let un = self.clone();
        *self = mov.clone().bbg;
        un
    }

    fn unmove(&mut self, mov: &Self::UnMove) {
        *self = mov.clone()
    }

    fn gen_alg(&mut self, _mov: &Self::Move) -> AlgebraicMove {
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
                (false,true ,false) => 'r',
                (false,true ,true ) => 'q',
                (true ,true ,true ) => 'k',
                _ => '#'
            };

            bstr.push(if is_white {c.to_ascii_uppercase()} else {c});
            if i % 8 == 7 {
                bstr.push('\n');
            }
        }
        f.write_fmt(format_args!("{}", bstr))
    }
}

impl Display for BitBoardGame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.board))
    }
}

impl Display for BitBoardGameMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.bbg))
    }
}

impl Move for BitBoardGameMove {
    fn to_uci(&self) -> String {
        let ox = ('h' as u8 - (self.mov & 7) as u8) as char;
        let oy = ('1' as u8 + ((self.mov >> 3) & 7) as u8) as char;
        let nx = ('h' as u8 - ((self.mov >> 6) & 7) as u8) as char;
        let ny = ('1' as u8 + ((self.mov >> 9) & 7) as u8) as char;
        format!("{ox}{oy}{nx}{ny}, {:x}", self.mov)
    }
}

impl BitBoardGame {
    fn from_parts(board: BitBoard, turn: bool, white_qs: bool, white_ks: bool,
             black_qs: bool, black_ks: bool, ep: Option<u8>) -> Self {
        Self { board, turn, white_qs, white_ks, black_qs, black_ks, ep }
    }
}

struct GenericMoveGenerator {
    next: Vec<BitBoardGameMove>
}

impl OnMove for GenericMoveGenerator {
    fn on_move<const TURN: bool, const WQ: bool,
            const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard, from: u8, to: u8) {
        let mut b = me.clone();
        b.mov(from, to);
        let next_state = BitBoardGame::from_parts(b, !TURN, WQ, WK, BQ, BK, None);
        let next_move = ((to as u16) << 6) + from as u16;
        let next_bbgm = BitBoardGameMove{mov: next_move, bbg: next_state};
        self.next.push(next_bbgm);
    }

    fn on_rook_move<const TURN: bool, const WQ: bool,
            const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard, from: u8, to: u8) {
        let mut b = me.clone();
        b.mov(from, to);
        let next_state = BitBoardGame::from_parts(b, !TURN, WQ && from != 7,
                WK && from != 0, BQ && from != 63, BK && from != 56, None);
        let next_move = ((to as u16) << 6) + from as u16;
        let next_bbgm = BitBoardGameMove{mov: next_move, bbg: next_state};
        self.next.push(next_bbgm);
    }

    fn on_king_move<const TURN: bool, const WQ: bool,
            const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard, from: u8, to: u8) {
        let mut b = me.clone();
        b.mov(from, to);
        let next_state = BitBoardGame::from_parts(b, !TURN, WQ && !TURN, WK && !TURN, BQ && TURN, BK && TURN, None);
        let next_move = ((to as u16) << 6) + from as u16;
        let next_bbgm = BitBoardGameMove{mov: next_move, bbg: next_state};
        self.next.push(next_bbgm);
    }

    fn on_ep_move<const TURN: bool, const WQ: bool,
            const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard, from: u8, to: u8) {
        let mut b = me.clone();
        b.mov(from, to);
        if TURN {
            b.clear(to - 8);
        }
        else {
            b.clear(to + 8);
        }
        let next_state = BitBoardGame::from_parts(b, !TURN, WQ, WK, BQ, BK, None);
        let next_move = ((to as u16) << 6) + from as u16;
        let next_bbgm = BitBoardGameMove{mov: next_move, bbg: next_state};
        self.next.push(next_bbgm);
    }

    fn on_qs_castle<const TURN: bool, const WQ: bool,
            const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard) {
        let mut b = me.clone();
        if TURN {
            b.mov(7, 4);
            b.mov(3, 5);
            let next_state = BitBoardGame::from_parts(b, !TURN, false, false, BQ, BK, None);
            let next_move = (5 << 6) + 3;
            let next_bbgm = BitBoardGameMove{mov: next_move, bbg: next_state};
            self.next.push(next_bbgm);
        }
        else {
            b.mov(63, 60);
            b.mov(59, 61);
            let next_state = BitBoardGame::from_parts(b, !TURN, WQ, WK, false, false, None);
            let next_move = (61 << 6) + 59;
            let next_bbgm = BitBoardGameMove{mov: next_move, bbg: next_state};
            self.next.push(next_bbgm);
        }
    }

    fn on_ks_castle<const TURN: bool, const WQ: bool,
        const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard) {
        let mut b = me.clone();
        if TURN {
            b.mov(0, 2);
            b.mov(3, 1);
            let next_state = BitBoardGame::from_parts(b, !TURN, false, false, BQ, BK, None);
            let next_move = (1 << 6) + 3;
            let next_bbgm = BitBoardGameMove{mov: next_move, bbg: next_state};
            self.next.push(next_bbgm);
        }
        else {
            b.mov(56, 58);
            b.mov(59, 57);
            let next_state = BitBoardGame::from_parts(b, !TURN, WQ, WK, false, false, None);
            let next_move = (57 << 6) + 59;
            let next_bbgm = BitBoardGameMove{mov: next_move, bbg: next_state};
            self.next.push(next_bbgm);
        }
    }

    fn on_pawn_push2<const TURN: bool, const WQ: bool,
        const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard, from: u8) {
        let mut b = me.clone();
        // println!("p2 {from}");
        if TURN {
            b.mov(from, from + 16);
            let next_state = BitBoardGame::from_parts(b, !TURN, WQ, WK, BQ, BK, Some(from + 8));
            let next_move = ((from as u16 + 16) << 6) + from as u16;
            let next_bbgm = BitBoardGameMove{mov: next_move, bbg: next_state};
            self.next.push(next_bbgm);
        }
        else {
            b.mov(from, from - 16);
            let next_state = BitBoardGame::from_parts(b, !TURN, WQ, WK, BQ, BK, Some(from - 8));
            let next_move = ((from as u16 - 16) << 6) + from as u16;
            let next_bbgm = BitBoardGameMove{mov: next_move, bbg: next_state};
            self.next.push(next_bbgm);
        }
    }
}
