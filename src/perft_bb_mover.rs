use crate::bit_board::{BitBoard, OnMove};

pub struct PerftMove {
    pub depth_target: u64,
    pub depth: u64,
    pub counter: u64,
pub struct PerftMove {
    pub depth_target: u64,
    pub depth: u64,
    pub counter: u64,
}

impl OnMove for PerftMove {
    fn on_move<const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>(
        &mut self,
        turn: bool,
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
            match (
                from != 7 && to != 7 && WQ,
                from != 0 && to != 0 && WK,
                from != 63 && to != 63 && BQ,
                from != 56 && to != 56 && BK,
            ) {
                (true, true, true, true) => {
                    b.gen_moves::<true, true, true, true, Self>(!turn, self)
                }
                (true, true, true, false) => {
                    b.gen_moves::<true, true, true, false, Self>(!turn, self)
                }
                (true, true, false, true) => {
                    b.gen_moves::<true, true, false, true, Self>(!turn, self)
                }
                (true, true, false, false) => {
                    b.gen_moves::<true, true, false, false, Self>(!turn, self)
                }
                (true, false, true, true) => {
                    b.gen_moves::<true, false, true, true, Self>(!turn, self)
                }
                (true, false, true, false) => {
                    b.gen_moves::<true, false, true, false, Self>(!turn, self)
                }
                (true, false, false, true) => {
                    b.gen_moves::<true, false, false, true, Self>(!turn, self)
                }
                (true, false, false, false) => {
                    b.gen_moves::<true, false, false, false, Self>(!turn, self)
                }
                (false, true, true, true) => {
                    b.gen_moves::<false, true, true, true, Self>(!turn, self)
                }
                (false, true, true, false) => {
                    b.gen_moves::<false, true, true, false, Self>(!turn, self)
                }
                (false, true, false, true) => {
                    b.gen_moves::<false, true, false, true, Self>(!turn, self)
                }
                (false, true, false, false) => {
                    b.gen_moves::<false, true, false, false, Self>(!turn, self)
                }
                (false, false, true, true) => {
                    b.gen_moves::<false, false, true, true, Self>(!turn, self)
                }
                (false, false, true, false) => {
                    b.gen_moves::<false, false, true, false, Self>(!turn, self)
                }
                (false, false, false, true) => {
                    b.gen_moves::<false, false, false, true, Self>(!turn, self)
                }
                (false, false, false, false) => {
                    b.gen_moves::<false, false, false, false, Self>(!turn, self)
                }
            }
        }
        self.depth -= 1;
    }

    fn on_king_move<const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>(
        &mut self,
        turn: bool,
        me: &BitBoard,
        from: u8,
        to: u8,
    ) {
        self.depth += 1;
        if self.depth == self.depth_target {
            self.counter += 1;
        } else {
            let mut b = me.clone();
            b.mov(from, to);
            b.gen_moves::<false, false, BQ, BK, Self>(!turn, self);
        }
        self.depth -= 1;
    }

    fn on_ep_move<const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>(
        &mut self,
        turn: bool,
        me: &BitBoard,
        from: u8,
        to: u8,
    ) {
        self.depth += 1;
        if self.depth == self.depth_target {
            self.counter += 1;
        } else {
            let mut b = me.clone();
            b.mov(from, to);
            if turn {
                b.clear(to - 8);
            } else {
                b.clear(to + 8);
            }
            b.gen_moves::<WQ, WK, BQ, BK, Self>(!turn, self);
        }
        self.depth -= 1;
    }

    fn on_qs_castle<const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>(
        &mut self,
        turn: bool,
        me: &BitBoard,
    ) {
        self.depth += 1;
        if self.depth == self.depth_target {
            self.counter += 1;
        } else {
            let mut b = me.clone();
            if turn {
                b.mov(7, 4);
                b.mov(3, 5);
                b.gen_moves::<false, false, BQ, BK, Self>(!turn, self);
            } else {
                b.mov(63, 60);
                b.mov(59, 61);
                b.gen_moves::<WQ, WK, false, false, Self>(!turn, self);
            }
        }
        self.depth -= 1;
    }

    fn on_ks_castle<const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>(
        &mut self,
        turn: bool,
        me: &BitBoard,
    ) {
        self.depth += 1;
        if self.depth == self.depth_target {
            self.counter += 1;
        } else {
            let mut b = me.clone();
            if turn {
                b.mov(0, 2);
                b.mov(3, 1);
                b.gen_moves::<false, false, BQ, BK, Self>(!turn, self);
            } else {
                b.mov(56, 58);
                b.mov(59, 57);
                b.gen_moves::<WQ, WK, false, false, Self>(!turn, self);
            }
        }
        self.depth -= 1;
    }

    fn on_pawn_push2<const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>(
        &mut self,
        turn: bool,
        me: &BitBoard,
        from: u8,
    ) {
        self.depth += 1;
        if self.depth == self.depth_target {
            self.counter += 1;
        } else {
            let mut b = me.clone();
            if turn {
                b.mov(from, from + 16);
                b.gen_moves_with_ep::<WQ, WK, BQ, BK, Self>(!turn, self, from + 8);
            } else {
                b.mov(from, from - 16);
                b.gen_moves_with_ep::<WQ, WK, BQ, BK, Self>(!turn, self, from - 8);
            }
        }
        self.depth -= 1;
    }

    fn on_promotion<const WQ: bool, const WK: bool, const BQ: bool, const BK: bool>(
        &mut self,
        _turn: bool,
        _me: &BitBoard,
        _from: u8,
        _to: u8,
        _piece: u8,
    ) {
        todo!()
    }
}
