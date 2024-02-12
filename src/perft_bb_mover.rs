use crate::bit_board::{BitBoard, OnMove};

pub struct PerftMove {
    pub depth_target: u64,
    pub depth: u64,
    pub counter: u64,
}

impl OnMove for PerftMove {
    fn on_move<const TURN: bool, const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>(
        &mut self,
        me: &BitBoard,
        from: u8,
        to: u8,
    ) {
        self.depth += 1;
        if self.depth == self.depth_target {
            self.counter += 1;
        } else {
            let mut b = me.clone();
            let mask = (1u64 << from) | (1u64 << to);
            b.mov(from, to);
            match (!TURN, mask & (1u64 << 7) != 0 && WQ, mask & (1u64 << 0) != 0 && WK, mask & (1u64 << 63) != 0 && BQ, mask & (1u64 << 56) != 0 && BK) {
                (true , true , true , true , true ) => b.gen_moves::<true , true , true , true , true , Self>(self),
                (true , true , true , true , false) => b.gen_moves::<true , true , true , true , false, Self>(self),
                (true , true , true , false, true ) => b.gen_moves::<true , true , true , false, true , Self>(self),
                (true , true , true , false, false) => b.gen_moves::<true , true , true , false, false, Self>(self),
                (true , true , false, true , true ) => b.gen_moves::<true , true , false, true , true , Self>(self),
                (true , true , false, true , false) => b.gen_moves::<true , true , false, true , false, Self>(self),
                (true , true , false, false, true ) => b.gen_moves::<true , true , false, false, true , Self>(self),
                (true , true , false, false, false) => b.gen_moves::<true , true , false, false, false, Self>(self),
                (true , false, true , true , true ) => b.gen_moves::<true , false, true , true , true , Self>(self),
                (true , false, true , true , false) => b.gen_moves::<true , false, true , true , false, Self>(self),
                (true , false, true , false, true ) => b.gen_moves::<true , false, true , false, true , Self>(self),
                (true , false, true , false, false) => b.gen_moves::<true , false, true , false, false, Self>(self),
                (true , false, false, true , true ) => b.gen_moves::<true , false, false, true , true , Self>(self),
                (true , false, false, true , false) => b.gen_moves::<true , false, false, true , false, Self>(self),
                (true , false, false, false, true ) => b.gen_moves::<true , false, false, false, true , Self>(self),
                (true , false, false, false, false) => b.gen_moves::<true , false, false, false, false, Self>(self),
                (false, true , true , true , true ) => b.gen_moves::<false, true , true , true , true , Self>(self),
                (false, true , true , true , false) => b.gen_moves::<false, true , true , true , false, Self>(self),
                (false, true , true , false, true ) => b.gen_moves::<false, true , true , false, true , Self>(self),
                (false, true , true , false, false) => b.gen_moves::<false, true , true , false, false, Self>(self),
                (false, true , false, true , true ) => b.gen_moves::<false, true , false, true , true , Self>(self),
                (false, true , false, true , false) => b.gen_moves::<false, true , false, true , false, Self>(self),
                (false, true , false, false, true ) => b.gen_moves::<false, true , false, false, true , Self>(self),
                (false, true , false, false, false) => b.gen_moves::<false, true , false, false, false, Self>(self),
                (false, false, true , true , true ) => b.gen_moves::<false, false, true , true , true , Self>(self),
                (false, false, true , true , false) => b.gen_moves::<false, false, true , true , false, Self>(self),
                (false, false, true , false, true ) => b.gen_moves::<false, false, true , false, true , Self>(self),
                (false, false, true , false, false) => b.gen_moves::<false, false, true , false, false, Self>(self),
                (false, false, false, true , true ) => b.gen_moves::<false, false, false, true , true , Self>(self),
                (false, false, false, true , false) => b.gen_moves::<false, false, false, true , false, Self>(self),
                (false, false, false, false, true ) => b.gen_moves::<false, false, false, false, true , Self>(self),
                (false, false, false, false, false) => b.gen_moves::<false, false, false, false, false, Self>(self),
            }
        }
        self.depth -= 1;
    }

    fn on_king_move<const TURN: bool, const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>
        (&mut self, me: &BitBoard, from: u8, to: u8) {
        self.depth += 1;
        if self.depth == self.depth_target {
            self.counter += 1;
        } else {
            let mut b = me.clone();
            b.mov(from, to);
            if TURN {
                b.gen_moves::<false, false, false, BQ, BK, Self>(self);
            } else {
                b.gen_moves::<true, WQ, WK, false, false, Self>(self);
            }
        }
        self.depth -= 1;
    }

    fn on_ep_move<const TURN: bool, const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>
        (&mut self, me: &BitBoard, from: u8, to: u8) {
        self.depth += 1;
        if self.depth == self.depth_target {
            self.counter += 1;
        } else {
            let mut b = me.clone();
            b.mov(from, to);
            if TURN {
                b.clear(to - 8);
            } else {
                b.clear(to + 8);
            }
            b.gen_moves::<false, WQ, WK, BQ, BK, Self>(self);
        }
        self.depth -= 1;
    }

    fn on_qs_castle< const TURN: bool, const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>
        (&mut self, me: &BitBoard) {
        self.depth += 1;
        if self.depth == self.depth_target {
            self.counter += 1;
        } else {
            let mut b = me.clone();
            if TURN {
                b.mov(7, 4);
                b.mov(3, 5);
                b.gen_moves::<false, false, false, BQ, BK, Self>(self);
            } else {
                b.mov(63, 60);
                b.mov(59, 61);
                b.gen_moves::<true, WQ, WK, false, false, Self>(self);
            }
        }
        self.depth -= 1;
    }

    fn on_ks_castle<
        const TURN: bool,
        const WQ: bool,
        const WK: bool,
        const BQ: bool,
        const BK: bool,
    >(
        &mut self,
        me: &BitBoard,
    ) {
        self.depth += 1;
        if self.depth == self.depth_target {
            self.counter += 1;
        } else {
            let mut b = me.clone();
            if TURN {
                b.mov(0, 2);
                b.mov(3, 1);
                b.gen_moves::<false, false, false, BQ, BK, Self>(self);
            } else {
                b.mov(56, 58);
                b.mov(59, 57);
                b.gen_moves::<true, WQ, WK, false, false, Self>(self);
            }
        }
        self.depth -= 1;
    }

    fn on_pawn_push2<const TURN: bool, const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>
        (&mut self, me: &BitBoard, from: u8) {
        self.depth += 1;
        if self.depth == self.depth_target {
            self.counter += 1;
        } else {
            let mut b = me.clone();
            if TURN {
                b.mov(from, from + 16);
                b.gen_moves_with_ep::<false, WQ, WK, BQ, BK, Self>(self, from + 8);
            } else {
                b.mov(from, from - 16);
                b.gen_moves_with_ep::<true, WQ, WK, BQ, BK, Self>(self, from - 8);
            }
        }
        self.depth -= 1;
    }

    fn on_promotion<const TURN: bool, const WQ: bool,
        const WK: bool, const BQ: bool, const BK: bool>(&mut self, _me: &BitBoard, _from: u8, _to: u8, _piece: u8) {
        todo!()
    }
}
