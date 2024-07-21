use std::fmt::Display;

use crate::{
    game::{ChessGame, Move},
    notation::AlgebraicMove,
};

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
    _packed: u16,
}

pub struct BoolExists<const _N: bool> {}

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
    board: [u64; 4],
}

#[derive(Clone)]
pub struct BitBoardGameMove {
    mov: u16,
    bbg: BitBoardGame,
}

pub trait OnMove {
    fn on_move<const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>(
        &mut self,
        turn: bool,
        me: &BitBoard,
        from: u8,
        to: u8,
    );
    fn on_king_move<const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>(
        &mut self,
        turn: bool,
        me: &BitBoard,
        from: u8,
        to: u8,
    );
    fn on_ep_move<const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>(
        &mut self,
        turn: bool,
        me: &BitBoard,
        from: u8,
        to: u8,
    );
    fn on_pawn_push2<const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>(
        &mut self,
        turn: bool,
        me: &BitBoard,
        from: u8,
    );
    fn on_promotion<const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>(
        &mut self,
        turn: bool,
        me: &BitBoard,
        from: u8,
        to: u8,
        piece: u8,
    );
    fn on_qs_castle<const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>(
        &mut self,
        turn: bool,
        me: &BitBoard,
    );
    fn on_ks_castle<const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>(
        &mut self,
        turn: bool,
        me: &BitBoard,
    );
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

    // place a piece on the board
    #[inline(always)]
    pub fn set(&mut self, square: u8, piece: u8) {
        self.clear(square);
        self.board
            .iter_mut()
            .enumerate()
            .for_each(|(i, v)| *v |= ((piece as u64 >> i) & 1) << square);
    }

    /// 1 if white
    /// 0 if black or no piece
    #[inline(always)]
    pub const fn colour_mask(&self, turn: bool) -> u64 {
        if turn {
            self.board[3]
        } else {
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
    pub const fn col_piece_mask(&self, turn: bool) -> u64 {
        self.piece_mask() & self.colour_mask(turn)
    }

    #[inline(always)]
    pub const fn enemy_or_empty(&self, turn: bool) -> u64 {
        !self.col_piece_mask(turn)
    }

    /// 1 if piece (incl. special enpassant square)
    /// 0 if no piece
    #[inline(always)]
    pub const fn piece_special_mask(&self) -> u64 {
        self.board[0] | self.board[1] | self.board[2] | self.board[3]
    }

    #[inline(always)]
    pub const fn sliding_mask<const SHIFT_LIFT: bool>(
        pieces: u64,
        step: u8,
        colision: u64,
        side_mask: u64,
    ) -> u64 {
        let mut mask = match SHIFT_LIFT {
            true => (pieces << step) & !side_mask,
            false => (pieces >> step) & !side_mask,
        };
        let mut i = 0;
        while i != 6 {
            mask |= match SHIFT_LIFT {
                true => ((mask & !colision) << step) & !side_mask,
                false => ((mask & !colision) >> step) & !side_mask,
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
    pub const fn col_pawn_mask(&self, turn: bool) -> u64 {
        self.pawn_mask() & self.colour_mask(turn)
    }

    #[inline(always)]
    pub const fn pawn_like_attack_mask(&self, turn: bool, pieces: u64) -> u64 {
        if turn {
            ((pieces << 9) & !Self::RIGHT_SIDE) | ((pieces << 7) & !Self::LEFT_SIDE)
        } else {
            ((pieces >> 7) & !Self::RIGHT_SIDE) | ((pieces >> 9) & !Self::LEFT_SIDE)
        }
    }

    /// colour 0 = white, u64::MAX = black
    /// 1 if colour pawn
    /// 0 if no colour pawn
    #[inline(always)]
    pub const fn pawn_attack_mask(&self, turn: bool) -> u64 {
        let pawns = self.col_pawn_mask(turn);
        self.pawn_like_attack_mask(turn, pawns)
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
    pub const fn col_knight_mask(&self, turn: bool) -> u64 {
        self.knight_mask() & (self.colour_mask(turn))
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

    /// colour 0 = white, u64::MAX = black
    /// 1 if colour knight can attack
    /// 0 if no colour knight cant attack
    #[inline(always)]
    pub const fn knight_attack_mask(&self, turn: bool) -> u64 {
        let knights = self.col_knight_mask(turn);
        self.knight_like_attack_mask(knights)
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
    pub const fn col_diagonal_mask(&self, turn: bool) -> u64 {
        self.diagonal_mask() & (self.colour_mask(turn))
    }

    #[inline(always)]
    pub const fn diagonal_like_attack_mask(&self, pieces: u64) -> u64 {
        let blockers = self.piece_mask();
        let ur = Self::sliding_mask::<true>(pieces, 7, blockers, Self::LEFT_SIDE);
        let ul = Self::sliding_mask::<true>(pieces, 9, blockers, Self::RIGHT_SIDE);
        let dr = Self::sliding_mask::<false>(pieces, 9, blockers, Self::LEFT_SIDE);
        let dl = Self::sliding_mask::<false>(pieces, 7, blockers, Self::RIGHT_SIDE);
        ur | ul | dr | dl
    }

    /// colour 0 = white, u64::MAX = black
    /// 1 if colour bishop can attack
    /// 0 if no colour bishop cant attack
    /// Note: a queen is a bishop
    #[inline(always)]
    pub const fn diagonal_attack_mask(&self, turn: bool) -> u64 {
        let bishops = self.col_diagonal_mask(turn);
        self.diagonal_like_attack_mask(bishops)
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
    pub const fn col_ortho_mask(&self, turn: bool) -> u64 {
        self.ortho_mask() & (self.colour_mask(turn))
    }

    #[inline(always)]
    pub const fn ortho_like_attack_mask(&self, pieces: u64) -> u64 {
        let blockers = self.piece_mask();
        let r = Self::sliding_mask::<true>(pieces, 1, blockers, Self::RIGHT_SIDE);
        let l = Self::sliding_mask::<false>(pieces, 1, blockers, Self::LEFT_SIDE);
        let u = Self::sliding_mask::<true>(pieces, 8, blockers, 0);
        let d = Self::sliding_mask::<false>(pieces, 8, blockers, 0);
        r | l | u | d
    }

    /// colour 0 = white, u64::MAX = black
    /// 1 if colour rook can attack
    /// 0 if no colour rook cant attack
    /// Note: a queen is a rook
    #[inline(always)]
    pub const fn ortho_attack_mask(&self, turn: bool) -> u64 {
        let rooks = self.col_ortho_mask(turn);
        self.ortho_like_attack_mask(rooks)
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
    pub const fn col_king_mask(&self, turn: bool) -> u64 {
        self.king_mask() & (self.colour_mask(turn))
    }

    /// colour 0 = white, u64::MAX = black
    /// 1 if colour king can attack
    /// 0 if no colour king cant attack
    #[inline(always)]
    pub const fn king_attack_mask(&self, turn: bool) -> u64 {
        let kings = self.col_king_mask(turn);
        let u = kings << 8;
        let d = kings >> 8;
        let mast = kings | u | d;
        ((mast >> 1) & !Self::LEFT_SIDE) | ((mast << 1) & !Self::RIGHT_SIDE) | u | d
    }

    #[inline(always)]
    pub const fn attack_mask(&self, turn: bool) -> u64 {
        self.pawn_attack_mask(turn)
            | self.knight_attack_mask(turn)
            | self.diagonal_attack_mask(turn)
            | self.ortho_attack_mask(turn)
            | self.king_attack_mask(turn)
    }

    #[inline(always)]
    pub const fn hor_check_mask(&self, turn: bool) -> u64 {
        let kings = self.col_king_mask(turn);
        let pieces = self.piece_mask();
        let other_ortho = self.col_ortho_mask(!turn);

        let mut mask = u64::MAX;

        let r1 = Self::sliding_mask::<false>(kings, 1, pieces, Self::LEFT_SIDE);
        if r1 & other_ortho != 0 {
            mask &= r1;
        }

        let r1 = Self::sliding_mask::<true>(kings, 1, pieces, Self::RIGHT_SIDE);
        if r1 & other_ortho != 0 {
            mask &= r1;
        }

        mask
    }

    #[inline(always)]
    pub const fn vert_check_mask(&self, turn: bool) -> u64 {
        let kings = self.col_king_mask(turn);
        let pieces = self.piece_mask();
        let other_ortho = self.col_ortho_mask(!turn);

        let mut mask = u64::MAX;

        let r1 = Self::sliding_mask::<true>(kings, 8, pieces, 0);
        if r1 & other_ortho != 0 {
            mask &= r1;
        }

        let r1 = Self::sliding_mask::<false>(kings, 8, pieces, 0);
        if r1 & other_ortho != 0 {
            mask &= r1;
        }

        mask
    }

    #[inline(always)]
    pub const fn lr_check_mask(&self, turn: bool) -> u64 {
        let kings = self.col_king_mask(turn);
        let pieces = self.piece_mask();
        let other_diag = self.col_diagonal_mask(!turn);

        let mut mask = u64::MAX;

        let r1 = Self::sliding_mask::<true>(kings, 7, pieces, Self::LEFT_SIDE);
        if r1 & other_diag != 0 {
            mask &= r1;
        }

        let r1 = Self::sliding_mask::<false>(kings, 7, pieces, Self::RIGHT_SIDE);
        if r1 & other_diag != 0 {
            mask &= r1;
        }

        mask
    }

    #[inline(always)]
    pub const fn rl_check_mask(&self, turn: bool) -> u64 {
        let kings = self.col_king_mask(turn);
        let pieces = self.piece_mask();
        let other_diag = self.col_diagonal_mask(!turn);

        let mut mask = u64::MAX;

        let r1 = Self::sliding_mask::<false>(kings, 9, pieces, Self::LEFT_SIDE);
        if r1 & other_diag != 0 {
            mask &= r1;
        }

        let r1 = Self::sliding_mask::<true>(kings, 9, pieces, Self::RIGHT_SIDE);
        if r1 & other_diag != 0 {
            mask &= r1;
        }

        mask
    }

    #[inline(always)]
    pub const fn check_mask(&self, turn: bool) -> u64 {
        let mut mask = u64::MAX;
        mask &= self.hor_check_mask(turn)
            & self.vert_check_mask(turn)
            & self.lr_check_mask(turn)
            & self.rl_check_mask(turn);

        let kings = self.col_king_mask(turn);

        let other_knights = Self::col_knight_mask(self, !turn);
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
            mask &= knights_r1 >> 17;
        }
        if (knights_r1 << 15) & other_knights != 0 {
            mask &= knights_r1 << 15;
        }
        if (knights_l1 >> 15) & other_knights != 0 {
            mask &= knights_l1 >> 15;
        }
        if (knights_l1 << 17) & other_knights != 0 {
            mask &= knights_l1 << 17;
        }
        if (knights_l2 >> 6) & other_knights != 0 {
            mask &= knights_l2 >> 6;
        }
        if (knights_l2 << 10) & other_knights != 0 {
            mask &= knights_l2 << 10;
        }

        let other_pawns = Self::col_pawn_mask(self, !turn);

        if turn {
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
    pub const fn vert_pin_mask(&self, turn: bool) -> u64 {
        let kings = self.col_king_mask(turn);
        let pieces = self.piece_mask();
        let other_ortho = self.col_ortho_mask(!turn);

        let mut mask = 0;
        let r1 = Self::sliding_mask::<true>(kings, 8, pieces, 0);
        let r2 = Self::sliding_mask::<true>(r1 & pieces, 8, pieces, 0);
        if r2 & other_ortho != 0 {
            mask |= r1 | r2;
        }

        let r1 = Self::sliding_mask::<false>(kings, 8, pieces, 0);
        let r2 = Self::sliding_mask::<false>(r1 & pieces, 8, pieces, 0);
        if r2 & other_ortho != 0 {
            mask |= r1 | r2;
        }

        mask
    }

    #[inline(always)]
    pub const fn hor_pin_mask(&self, turn: bool) -> u64 {
        let kings = self.col_king_mask(turn);
        let pieces = self.piece_mask();
        let other_ortho = self.col_ortho_mask(!turn);

        let r1 = Self::sliding_mask::<false>(kings, 1, pieces, Self::LEFT_SIDE);
        let r2 = Self::sliding_mask::<false>(r1 & pieces, 1, pieces, Self::LEFT_SIDE);
        let mut mask = 0;
        if r2 & other_ortho != 0 {
            mask |= r1 | r2;
        }

        let r1 = Self::sliding_mask::<true>(kings, 1, pieces, Self::RIGHT_SIDE);
        let r2 = Self::sliding_mask::<true>(r1 & pieces, 1, pieces, Self::RIGHT_SIDE);
        if r2 & other_ortho != 0 {
            mask |= r1 | r2;
        }

        mask
    }

    // Horizontal pin that goes through two pieces
    #[inline(always)]
    pub const fn hor_pin_mask2(&self, turn: bool) -> u64 {
        let kings = self.col_king_mask(turn);
        let blockers = self.piece_mask();
        let other_ortho = self.col_ortho_mask(!turn);

        let r1 = Self::sliding_mask::<false>(kings, 1, blockers, Self::LEFT_SIDE);
        let r2 = Self::sliding_mask::<false>(r1 & blockers, 1, blockers, Self::LEFT_SIDE);
        let r3 = Self::sliding_mask::<false>(r2 & blockers, 1, blockers, Self::LEFT_SIDE);
        let mut mask = 0;
        if r3 & other_ortho != 0 {
            mask |= r1 | r2 | r3;
        }

        let r1 = Self::sliding_mask::<true>(kings, 1, blockers, Self::RIGHT_SIDE);
        let r2 = Self::sliding_mask::<true>(r1 & blockers, 1, blockers, Self::RIGHT_SIDE);
        let r3 = Self::sliding_mask::<true>(r2 & blockers, 1, blockers, Self::RIGHT_SIDE);
        if r3 & other_ortho != 0 {
            mask |= r1 | r2 | r3;
        }

        mask
    }

    #[inline(always)]
    pub const fn ortho_pin_mask(&self, turn: bool) -> u64 {
        self.hor_pin_mask(turn) | self.vert_pin_mask(turn)
    }

    pub const fn lr_pin_mask(&self, turn: bool) -> u64 {
        let kings = self.col_king_mask(turn);
        let pieces = self.piece_mask();
        let other_diag = self.col_diagonal_mask(!turn);

        let mut mask = 0;
        let r1 = Self::sliding_mask::<true>(kings, 7, pieces, Self::LEFT_SIDE);
        let r2 = Self::sliding_mask::<true>(r1 & pieces, 7, pieces, Self::LEFT_SIDE);
        if r2 & other_diag != 0 {
            mask |= r1 | r2;
        }

        let r1 = Self::sliding_mask::<false>(kings, 7, pieces, Self::RIGHT_SIDE);
        let r2 = Self::sliding_mask::<false>(r1 & pieces, 7, pieces, Self::RIGHT_SIDE);
        if r2 & other_diag != 0 {
            mask |= r1 | r2;
        }

        mask
    }

    pub const fn rl_pin_mask(&self, turn: bool) -> u64 {
        let kings = self.col_king_mask(turn);
        let pieces = self.piece_mask();
        let other_diag = self.col_diagonal_mask(!turn);

        let mut mask = 0;
        let r1 = Self::sliding_mask::<false>(kings, 9, pieces, Self::LEFT_SIDE);
        let r2 = Self::sliding_mask::<false>(r1 & pieces, 9, pieces, Self::LEFT_SIDE);
        if r2 & other_diag != 0 {
            mask |= r1 | r2;
        }

        let r1 = Self::sliding_mask::<true>(kings, 9, pieces, Self::RIGHT_SIDE);
        let r2 = Self::sliding_mask::<true>(r1 & pieces, 9, pieces, Self::RIGHT_SIDE);
        if r2 & other_diag != 0 {
            mask |= r1 | r2;
        }

        mask
    }

    pub const fn diagonal_pin_mask(&self, turn: bool) -> u64 {
        self.lr_pin_mask(turn) | self.rl_pin_mask(turn)
    }

    #[inline(always)]
    pub fn gen_pawn_moves<
        const WQ: bool,
        const WK: bool,
        const BQ: bool,
        const BK: bool,
        Mov: OnMove,
    >(
        &self,
        turn: bool,
        on_move: &mut Mov,
        sq: Option<u8>,
    ) {
        let check_mask = self.check_mask(turn);
        let hor_pins = self.hor_pin_mask(turn);
        let ortho_pins = self.ortho_pin_mask(turn);
        let lr_pins = self.lr_pin_mask(turn);
        let rl_pins = self.rl_pin_mask(turn);
        let diagonal_pins = self.diagonal_pin_mask(turn);
        let empty = !self.piece_mask();
        let empty_free = empty & check_mask;
        let ep = match sq {
            Some(x) => 1 << x,
            None => 0,
        };
        let enemy = self.col_piece_mask(!turn) & check_mask | ep;

        let base_pawns = self.col_pawn_mask(turn);
        let lr_pawns = base_pawns & !rl_pins & !ortho_pins;
        let up_pawns = base_pawns & !diagonal_pins & !hor_pins;
        let rl_pawns = base_pawns & !lr_pins & !ortho_pins;
        if turn {
            let mut up1 = (empty_free >> 8) & up_pawns;
            let mut up2 = (empty_free >> 16) & (empty >> 8) & up_pawns & (0xff << 8);
            let mut lr = (enemy >> 7) & lr_pawns & !Self::RIGHT_SIDE;
            let mut rl = (enemy >> 9) & rl_pawns & !Self::LEFT_SIDE;
            while up1 != 0 {
                let from_idx = up1.trailing_zeros() as u8;
                if from_idx >> 3 == 6 {
                    on_move.on_promotion::<WQ, WK, BQ, BK>(
                        turn,
                        self,
                        from_idx,
                        from_idx + 8,
                        0b1101,
                    );
                    on_move.on_promotion::<WQ, WK, BQ, BK>(
                        turn,
                        self,
                        from_idx,
                        from_idx + 8,
                        0b1001,
                    );
                    on_move.on_promotion::<WQ, WK, BQ, BK>(
                        turn,
                        self,
                        from_idx,
                        from_idx + 8,
                        0b1010,
                    );
                    on_move.on_promotion::<WQ, WK, BQ, BK>(
                        turn,
                        self,
                        from_idx,
                        from_idx + 8,
                        0b1011,
                    );
                } else {
                    on_move.on_move::<WQ, WK, BQ, BK>(turn, self, from_idx, from_idx + 8);
                }
                up1 &= up1 - 1;
            }
            while up2 != 0 {
                let from_idx = up2.trailing_zeros() as u8;
                on_move.on_pawn_push2::<WQ, WK, BQ, BK>(turn, self, from_idx);
                up2 &= up2 - 1;
            }
            while lr != 0 {
                let from_idx = lr.trailing_zeros() as u8;
                if empty & (1 << (from_idx + 7)) == 0 {
                    if from_idx >> 3 == 6 {
                        on_move.on_promotion::<WQ, WK, BQ, BK>(
                            turn,
                            self,
                            from_idx,
                            from_idx + 7,
                            0b1101,
                        );
                        on_move.on_promotion::<WQ, WK, BQ, BK>(
                            turn,
                            self,
                            from_idx,
                            from_idx + 7,
                            0b1001,
                        );
                        on_move.on_promotion::<WQ, WK, BQ, BK>(
                            turn,
                            self,
                            from_idx,
                            from_idx + 7,
                            0b1010,
                        );
                        on_move.on_promotion::<WQ, WK, BQ, BK>(
                            turn,
                            self,
                            from_idx,
                            from_idx + 7,
                            0b1011,
                        );
                    } else {
                        on_move.on_move::<WQ, WK, BQ, BK>(turn, self, from_idx, from_idx + 7);
                    }
                } else if self.hor_pin_mask2(turn) & (0xff << (8 * 4)) == 0
                    && check_mask & (1 << (from_idx - 1)) != 0
                {
                    on_move.on_ep_move::<WQ, WK, BQ, BK>(turn, self, from_idx, from_idx + 7);
                }
                lr &= lr - 1;
            }
            while rl != 0 {
                let from_idx = rl.trailing_zeros() as u8;
                if empty & (1 << (from_idx + 9)) == 0 {
                    if from_idx >> 3 == 6 {
                        on_move.on_promotion::<WQ, WK, BQ, BK>(
                            turn,
                            self,
                            from_idx,
                            from_idx + 9,
                            0b1101,
                        );
                        on_move.on_promotion::<WQ, WK, BQ, BK>(
                            turn,
                            self,
                            from_idx,
                            from_idx + 9,
                            0b1001,
                        );
                        on_move.on_promotion::<WQ, WK, BQ, BK>(
                            turn,
                            self,
                            from_idx,
                            from_idx + 9,
                            0b1010,
                        );
                        on_move.on_promotion::<WQ, WK, BQ, BK>(
                            turn,
                            self,
                            from_idx,
                            from_idx + 9,
                            0b1011,
                        );
                    } else {
                        on_move.on_move::<WQ, WK, BQ, BK>(turn, self, from_idx, from_idx + 9);
                    }
                } else if self.hor_pin_mask2(turn) & (0xff << (8 * 4)) == 0
                    && check_mask & (1 << (from_idx + 1)) != 0
                {
                    on_move.on_ep_move::<WQ, WK, BQ, BK>(turn, self, from_idx, from_idx + 9);
                }
                rl &= rl - 1;
            }
        } else {
            let mut up1 = (empty_free << 8) & up_pawns;
            let mut up2 = (empty_free << 16) & (empty << 8) & up_pawns & (0xff << (8 * 6));
            let mut lr = (enemy << 7) & lr_pawns & !Self::LEFT_SIDE;
            let mut rl = (enemy << 9) & rl_pawns & !Self::RIGHT_SIDE;
            while up1 != 0 {
                let from_idx = up1.trailing_zeros() as u8;
                if from_idx >> 3 == 1 {
                    on_move.on_promotion::<WQ, WK, BQ, BK>(
                        turn,
                        self,
                        from_idx,
                        from_idx - 8,
                        0b0101,
                    );
                    on_move.on_promotion::<WQ, WK, BQ, BK>(
                        turn,
                        self,
                        from_idx,
                        from_idx - 8,
                        0b0001,
                    );
                    on_move.on_promotion::<WQ, WK, BQ, BK>(
                        turn,
                        self,
                        from_idx,
                        from_idx - 8,
                        0b0010,
                    );
                    on_move.on_promotion::<WQ, WK, BQ, BK>(
                        turn,
                        self,
                        from_idx,
                        from_idx - 8,
                        0b0011,
                    );
                } else {
                    on_move.on_move::<WQ, WK, BQ, BK>(turn, self, from_idx, from_idx - 8);
                }
                up1 &= up1 - 1;
            }
            while up2 != 0 {
                let from_idx = up2.trailing_zeros() as u8;
                on_move.on_pawn_push2::<WQ, WK, BQ, BK>(turn, self, from_idx);
                up2 &= up2 - 1;
            }
            while lr != 0 {
                let from_idx = lr.trailing_zeros() as u8;
                if empty & (1 << (from_idx - 7)) == 0 {
                    if from_idx >> 3 == 1 {
                        on_move.on_promotion::<WQ, WK, BQ, BK>(
                            turn,
                            self,
                            from_idx,
                            from_idx - 7,
                            0b0101,
                        );
                        on_move.on_promotion::<WQ, WK, BQ, BK>(
                            turn,
                            self,
                            from_idx,
                            from_idx - 7,
                            0b0001,
                        );
                        on_move.on_promotion::<WQ, WK, BQ, BK>(
                            turn,
                            self,
                            from_idx,
                            from_idx - 7,
                            0b0010,
                        );
                        on_move.on_promotion::<WQ, WK, BQ, BK>(
                            turn,
                            self,
                            from_idx,
                            from_idx - 7,
                            0b0011,
                        );
                    } else {
                        on_move.on_move::<WQ, WK, BQ, BK>(turn, self, from_idx, from_idx - 7);
                    }
                } else if self.hor_pin_mask2(turn) & (0xff << (8 * 3)) == 0
                    && check_mask & (1 << (from_idx + 1)) != 0
                {
                    on_move.on_ep_move::<WQ, WK, BQ, BK>(turn, self, from_idx, from_idx - 7);
                }
                lr &= lr - 1;
            }
            while rl != 0 {
                let from_idx = rl.trailing_zeros() as u8;
                if empty & (1 << (from_idx - 9)) == 0 {
                    if from_idx >> 3 == 1 {
                        on_move.on_promotion::<WQ, WK, BQ, BK>(
                            turn,
                            self,
                            from_idx,
                            from_idx - 9,
                            0b0101,
                        );
                        on_move.on_promotion::<WQ, WK, BQ, BK>(
                            turn,
                            self,
                            from_idx,
                            from_idx - 9,
                            0b0001,
                        );
                        on_move.on_promotion::<WQ, WK, BQ, BK>(
                            turn,
                            self,
                            from_idx,
                            from_idx - 9,
                            0b0010,
                        );
                        on_move.on_promotion::<WQ, WK, BQ, BK>(
                            turn,
                            self,
                            from_idx,
                            from_idx - 9,
                            0b0011,
                        );
                    } else {
                        on_move.on_move::<WQ, WK, BQ, BK>(turn, self, from_idx, from_idx - 9);
                    }
                } else if self.hor_pin_mask2(turn) & (0xff << (8 * 3)) == 0
                    && check_mask & (1 << (from_idx - 1)) != 0
                {
                    on_move.on_ep_move::<WQ, WK, BQ, BK>(turn, self, from_idx, from_idx - 9);
                }
                rl &= rl - 1;
            }
        }
    }

    #[inline(always)]
    pub fn gen_knight_moves<
        const WQ: bool,
        const WK: bool,
        const BQ: bool,
        const BK: bool,
        Mov: OnMove,
    >(
        &self,
        turn: bool,
        on_move: &mut Mov,
    ) {
        let base_mask = self.enemy_or_empty(turn) & self.check_mask(turn);
        let ortho_pins = self.ortho_pin_mask(turn);
        let diagonal_pins = self.diagonal_pin_mask(turn);

        let mut knights = self.col_knight_mask(turn) & !ortho_pins & !diagonal_pins;
        while knights != 0 {
            let from_idx = knights.trailing_zeros() as u8;
            let mut to_mask = self.knight_like_attack_mask(1 << from_idx) & base_mask;
            while to_mask != 0 {
                let to_idx = to_mask.trailing_zeros() as u8;
                on_move.on_move::<WQ, WK, BQ, BK>(turn, self, from_idx, to_idx);
                to_mask &= to_mask - 1;
            }
            knights &= knights - 1;
        }
    }

    #[inline(always)]
    pub fn gen_diagonal_moves<
        const WQ: bool,
        const WK: bool,
        const BQ: bool,
        const BK: bool,
        Mov: OnMove,
    >(
        &self,
        turn: bool,
        on_move: &mut Mov,
    ) {
        let base_mask = self.enemy_or_empty(turn) & self.check_mask(turn);
        let ortho_pins = self.ortho_pin_mask(turn);
        let diagonal_pins = self.diagonal_pin_mask(turn);

        let mut free_bishops = self.col_diagonal_mask(turn) & !diagonal_pins & !ortho_pins;
        let mut pin_bishops = self.col_diagonal_mask(turn) & diagonal_pins;

        while free_bishops != 0 {
            let from_idx = free_bishops.trailing_zeros() as u8;
            let mut to_mask = self.diagonal_like_attack_mask(1 << from_idx) & base_mask;
            while to_mask != 0 {
                let to_idx = to_mask.trailing_zeros() as u8;
                on_move.on_move::<WQ, WK, BQ, BK>(turn, self, from_idx, to_idx);
                to_mask &= to_mask - 1;
            }
            free_bishops &= free_bishops - 1;
        }

        while pin_bishops != 0 {
            let from_idx = pin_bishops.trailing_zeros() as u8;
            let mut to_mask =
                self.diagonal_like_attack_mask(1 << from_idx) & base_mask & diagonal_pins;
            while to_mask != 0 {
                let to_idx = to_mask.trailing_zeros() as u8;
                on_move.on_move::<WQ, WK, BQ, BK>(turn, self, from_idx, to_idx);
                to_mask &= to_mask - 1;
            }
            pin_bishops &= pin_bishops - 1;
        }
    }

    #[inline(always)]
    pub fn gen_ortho_moves<
        const WQ: bool,
        const WK: bool,
        const BQ: bool,
        const BK: bool,
        Mov: OnMove,
    >(
        &self,
        turn: bool,
        on_move: &mut Mov,
    ) {
        let base_mask = self.enemy_or_empty(turn) & self.check_mask(turn);
        let ortho_pins = self.ortho_pin_mask(turn);
        let diagonal_pins = self.diagonal_pin_mask(turn);

        let mut free_rooks = self.col_ortho_mask(turn) & !diagonal_pins & !ortho_pins;
        let mut pin_rooks = self.col_ortho_mask(turn) & ortho_pins;

        while free_rooks != 0 {
            let from_idx = free_rooks.trailing_zeros() as u8;
            let mut to_mask = self.ortho_like_attack_mask(1 << from_idx) & base_mask;
            while to_mask != 0 {
                let to_idx = to_mask.trailing_zeros() as u8;
                on_move.on_move::<WQ, WK, BQ, BK>(turn, self, from_idx, to_idx);
                to_mask &= to_mask - 1;
            }
            free_rooks &= free_rooks - 1;
        }

        while pin_rooks != 0 {
            let from_idx = pin_rooks.trailing_zeros() as u8;
            let mut to_mask = self.ortho_like_attack_mask(1 << from_idx) & base_mask & ortho_pins;
            while to_mask != 0 {
                let to_idx = to_mask.trailing_zeros() as u8;
                on_move.on_move::<WQ, WK, BQ, BK>(turn, self, from_idx, to_idx);
                to_mask &= to_mask - 1;
            }
            pin_rooks &= pin_rooks - 1;
        }
    }

    #[inline(always)]
    pub fn gen_king_moves<
        const WQ: bool,
        const WK: bool,
        const BQ: bool,
        const BK: bool,
        Mov: OnMove,
    >(
        &self,
        turn: bool,
        on_move: &mut Mov,
    ) {
        let empty = !self.piece_mask();
        let other_attacks = self.attack_mask(!turn);
        let base_mask = self.enemy_or_empty(turn) & !other_attacks;
        let king = self.col_king_mask(turn);

        let from_idx = king.trailing_zeros() as u8;
        let mut to_mask = self.king_attack_mask(turn) & base_mask;

        if self.hor_check_mask(turn) != u64::MAX {
            to_mask &= !((!Self::LEFT_SIDE & (king >> 1) | !Self::RIGHT_SIDE & (king << 1))
                & !self.col_ortho_mask(!turn));
        }

        if self.vert_check_mask(turn) != u64::MAX {
            to_mask &= !(((king >> 8) | (king << 8)) & !self.col_ortho_mask(!turn));
        }

        if self.lr_check_mask(turn) != u64::MAX {
            to_mask &= !((!Self::RIGHT_SIDE & (king >> 7) | !Self::LEFT_SIDE & (king << 7))
                & !self.col_diagonal_mask(!turn));
        }

        if self.rl_check_mask(turn) != u64::MAX {
            to_mask &= !((!Self::LEFT_SIDE & (king >> 9) | !Self::RIGHT_SIDE & (king << 9))
                & !self.col_diagonal_mask(!turn));
        }

        while to_mask != 0 {
            let to_idx = to_mask.trailing_zeros() as u8;
            on_move.on_king_move::<WQ, WK, BQ, BK>(turn, self, from_idx, to_idx);
            to_mask &= to_mask - 1;
        }

        if WK && ((0b00000110 & empty) + 8) & !other_attacks == 0b00001110 {
            on_move.on_ks_castle::<WQ, WK, BQ, BK>(turn, self);
        }

        if BK && ((0b00000110 & (empty >> 56)) + 8) & (!other_attacks >> 56) == 0b00001110 {
            on_move.on_ks_castle::<WQ, WK, BQ, BK>(turn, self);
        }

        if WQ && ((0b01110000 & empty) >> 1) & !other_attacks == 0b00111000 {
            on_move.on_qs_castle::<WQ, WK, BQ, BK>(turn, self);
        }

        if BQ && ((0b01110000 & (empty >> 56)) >> 1) & (!other_attacks >> 56) == 0b00111000 {
            on_move.on_qs_castle::<WQ, WK, BQ, BK>(turn, self);
        }
    }

    #[inline(always)]
    pub fn gen_moves<
        const WQ: bool,
        const WK: bool,
        const BQ: bool,
        const BK: bool,
        Mov: OnMove,
    >(
        &self,
        turn: bool,
        on_move: &mut Mov,
        ep: Option<u8>,
    ) {
        self.gen_pawn_moves::<WQ, WK, BQ, BK, Mov>(turn, on_move, ep);
        self.gen_knight_moves::<WQ, WK, BQ, BK, Mov>(turn, on_move);
        self.gen_diagonal_moves::<WQ, WK, BQ, BK, Mov>(turn, on_move);
        self.gen_ortho_moves::<WQ, WK, BQ, BK, Mov>(turn, on_move);
        self.gen_king_moves::<WQ, WK, BQ, BK, Mov>(turn, on_move);
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
            _ => return None,
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
            Some(x) => Some(if turn { x + 40 } else { x + 16 }),
            _ => None,
        };

        let mut board: [u64; 4] = [0; 4];
        let mut counter = 0;
        for c in fenboard.replace('/', "").chars() {
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
                _ => return None,
            };
            piece_idx |= if c.is_ascii_uppercase() { 0b1000 } else { 0 };
            board
                .iter_mut()
                .enumerate()
                .for_each(|(i, v)| *v |= ((piece_idx >> i) & 1) << (63 - counter));
            counter += 1;
        }

        if counter == 64 {
            Some(BitBoardGame {
                board: BitBoard { board },
                turn,
                white_qs: white_qs_castle,
                white_ks: white_ks_castle,
                black_qs: black_qs_castle,
                black_ks: black_ks_castle,
                ep: enpassant,
            })
        } else {
            None
        }
    }

    fn decode_alg(&mut self, _mov: &AlgebraicMove) -> Self::Move {
        todo!()
    }

    fn moves(&self) -> Vec<Self::Move> {
        let mut genny = GenericMoveGenerator {
            next: Vec::with_capacity(240),
        };
        self.proc_movs(&mut genny);
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

impl BitBoardGame {
    pub fn proc_movs<MOV: OnMove>(&self, mov: &mut MOV) {
        let turn = self.turn;
        match (self.white_qs, self.white_ks, self.black_qs, self.black_ks) {
            (true, true, true, true) => self
                .board
                .gen_moves::<true, true, true, true, MOV>(turn, mov, self.ep),
            (true, true, true, false) => self
                .board
                .gen_moves::<true, true, true, false, MOV>(turn, mov, self.ep),
            (true, true, false, true) => self
                .board
                .gen_moves::<true, true, false, true, MOV>(turn, mov, self.ep),
            (true, true, false, false) => self
                .board
                .gen_moves::<true, true, false, false, MOV>(turn, mov, self.ep),
            (true, false, true, true) => self
                .board
                .gen_moves::<true, false, true, true, MOV>(turn, mov, self.ep),
            (true, false, true, false) => self
                .board
                .gen_moves::<true, false, true, false, MOV>(turn, mov, self.ep),
            (true, false, false, true) => self
                .board
                .gen_moves::<true, false, false, true, MOV>(turn, mov, self.ep),
            (true, false, false, false) => self
                .board
                .gen_moves::<true, false, false, false, MOV>(turn, mov, self.ep),
            (false, true, true, true) => self
                .board
                .gen_moves::<false, true, true, true, MOV>(turn, mov, self.ep),
            (false, true, true, false) => self
                .board
                .gen_moves::<false, true, true, false, MOV>(turn, mov, self.ep),
            (false, true, false, true) => self
                .board
                .gen_moves::<false, true, false, true, MOV>(turn, mov, self.ep),
            (false, true, false, false) => self
                .board
                .gen_moves::<false, true, false, false, MOV>(turn, mov, self.ep),
            (false, false, true, true) => self
                .board
                .gen_moves::<false, false, true, true, MOV>(turn, mov, self.ep),
            (false, false, true, false) => self
                .board
                .gen_moves::<false, false, true, false, MOV>(turn, mov, self.ep),
            (false, false, false, true) => self
                .board
                .gen_moves::<false, false, false, true, MOV>(turn, mov, self.ep),
            (false, false, false, false) => self
                .board
                .gen_moves::<false, false, false, false, MOV>(turn, mov, self.ep),
        }
    }
}

pub fn print_bitmask(mask: u64) {
    let mut bstr = String::from("");
    for i in 0..64 {
        if i % 8 == 0 {
            bstr.push('\n')
        }
        bstr.push(if (mask >> 63 - i) & 1 == 0 { '-' } else { 'X' });
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

            let c = match (
                self.board[2] & mask != 0,
                self.board[1] & mask != 0,
                self.board[0] & mask != 0,
            ) {
                (false, false, false) => {
                    if is_white {
                        '*'
                    } else {
                        '-'
                    }
                }
                (true, false, false) => 'p',
                (true, false, true) => 'n',
                (false, false, true) => 'b',
                (false, true, false) => 'r',
                (false, true, true) => 'q',
                (true, true, true) => 'k',
                _ => '#',
            };

            bstr.push(if is_white { c.to_ascii_uppercase() } else { c });
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
        format!("{ox}{oy}{nx}{ny}")
    }
}

impl BitBoardGame {
    fn from_parts(
        board: BitBoard,
        turn: bool,
        white_qs: bool,
        white_ks: bool,
        black_qs: bool,
        black_ks: bool,
        ep: Option<u8>,
    ) -> Self {
        Self {
            board,
            turn,
            white_qs,
            white_ks,
            black_qs,
            black_ks,
            ep,
        }
    }
}

struct GenericMoveGenerator {
    next: Vec<BitBoardGameMove>,
}

impl OnMove for GenericMoveGenerator {
    fn on_move<const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>(
        &mut self,
        turn: bool,
        me: &BitBoard,
        from: u8,
        to: u8,
    ) {
        let mut b = me.clone();
        b.mov(from, to);
        let next_state = BitBoardGame::from_parts(
            b,
            !turn,
            from != 7 && to != 7 && WQ,
            from != 0 && to != 0 && WK,
            from != 63 && to != 63 && BQ,
            from != 56 && to != 56 && BK,
            None,
        );
        let next_move = ((to as u16) << 6) + from as u16;
        let next_bbgm = BitBoardGameMove {
            mov: next_move,
            bbg: next_state,
        };
        self.next.push(next_bbgm);
    }

    fn on_king_move<const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>(
        &mut self,
        turn: bool,
        me: &BitBoard,
        from: u8,
        to: u8,
    ) {
        let mut b = me.clone();
        b.mov(from, to);
        let next_state = BitBoardGame::from_parts(
            b,
            !turn,
            WQ && !turn,
            WK && !turn,
            BQ && turn,
            BK && turn,
            None,
        );
        let next_move = ((to as u16) << 6) + from as u16;
        let next_bbgm = BitBoardGameMove {
            mov: next_move,
            bbg: next_state,
        };
        self.next.push(next_bbgm);
    }

    fn on_ep_move<const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>(
        &mut self,
        turn: bool,
        me: &BitBoard,
        from: u8,
        to: u8,
    ) {
        let mut b = me.clone();
        b.mov(from, to);
        if turn {
            b.clear(to - 8);
        } else {
            b.clear(to + 8);
        }
        let next_state = BitBoardGame::from_parts(b, !turn, WQ, WK, BQ, BK, None);
        let next_move = ((to as u16) << 6) + from as u16;
        let next_bbgm = BitBoardGameMove {
            mov: next_move,
            bbg: next_state,
        };
        self.next.push(next_bbgm);
    }

    fn on_qs_castle<const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>(
        &mut self,
        turn: bool,
        me: &BitBoard,
    ) {
        let mut b = me.clone();
        if turn {
            b.mov(7, 4);
            b.mov(3, 5);
            let next_state = BitBoardGame::from_parts(b, !turn, false, false, BQ, BK, None);
            let next_move = (5 << 6) + 3;
            let next_bbgm = BitBoardGameMove {
                mov: next_move,
                bbg: next_state,
            };
            self.next.push(next_bbgm);
        } else {
            b.mov(63, 60);
            b.mov(59, 61);
            let next_state = BitBoardGame::from_parts(b, !turn, WQ, WK, false, false, None);
            let next_move = (61 << 6) + 59;
            let next_bbgm = BitBoardGameMove {
                mov: next_move,
                bbg: next_state,
            };
            self.next.push(next_bbgm);
        }
    }

    fn on_ks_castle<const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>(
        &mut self,
        turn: bool,
        me: &BitBoard,
    ) {
        let mut b = me.clone();
        if turn {
            b.mov(0, 2);
            b.mov(3, 1);
            let next_state = BitBoardGame::from_parts(b, !turn, false, false, BQ, BK, None);
            let next_move = (1 << 6) + 3;
            let next_bbgm = BitBoardGameMove {
                mov: next_move,
                bbg: next_state,
            };
            self.next.push(next_bbgm);
        } else {
            b.mov(56, 58);
            b.mov(59, 57);
            let next_state = BitBoardGame::from_parts(b, !turn, WQ, WK, false, false, None);
            let next_move = (57 << 6) + 59;
            let next_bbgm = BitBoardGameMove {
                mov: next_move,
                bbg: next_state,
            };
            self.next.push(next_bbgm);
        }
    }

    fn on_pawn_push2<const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>(
        &mut self,
        turn: bool,
        me: &BitBoard,
        from: u8,
    ) {
        let mut b = me.clone();
        if turn {
            b.mov(from, from + 16);
            let next_state = BitBoardGame::from_parts(b, !turn, WQ, WK, BQ, BK, Some(from + 8));
            let next_move = ((from as u16 + 16) << 6) + from as u16;
            let next_bbgm = BitBoardGameMove {
                mov: next_move,
                bbg: next_state,
            };
            self.next.push(next_bbgm);
        } else {
            b.mov(from, from - 16);
            let next_state = BitBoardGame::from_parts(b, !turn, WQ, WK, BQ, BK, Some(from - 8));
            let next_move = ((from as u16 - 16) << 6) + from as u16;
            let next_bbgm = BitBoardGameMove {
                mov: next_move,
                bbg: next_state,
            };
            self.next.push(next_bbgm);
        }
    }

    fn on_promotion<const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>(
        &mut self,
        turn: bool,
        me: &BitBoard,
        from: u8,
        to: u8,
        piece: u8,
    ) {
        let mut b = me.clone();
        b.clear(from);
        b.set(to, piece);
        let next_state = BitBoardGame::from_parts(
            b,
            !turn,
            to != 7 && WQ,
            to != 0 && WK,
            to != 63 && BQ,
            to != 56 && BK,
            None,
        );
        let next_move = ((to as u16) << 6) + from as u16;
        let next_bbgm = BitBoardGameMove {
            mov: next_move,
            bbg: next_state,
        };
        self.next.push(next_bbgm);
    }
}
