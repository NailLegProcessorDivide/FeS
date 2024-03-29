use crate::bit_board::{BitBoard, OnMove};

struct PerftMove {
    depth_target: u64,
    depth: u64,
    counter: u64,
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
            b.mov(from, to);
            if TURN {
                b.gen_moves::<false, WQ, WK, BQ, BK, Self>(self);
            } else {
                b.gen_moves::<true, WQ, WK, BQ, BK, Self>(self);
            }
        }
        self.depth -= 1;
    }

    fn on_rook_move<
        const TURN: bool,
        const WQ: bool,
        const WK: bool,
        const BQ: bool,
        const BK: bool,
    >(
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
            b.mov(from, to);
            match from {
                0 => {
                    if TURN {
                        b.gen_moves::<false, false, WK, BQ, BK, Self>(self);
                    } else {
                        b.gen_moves::<true, false, WK, BQ, BK, Self>(self);
                    }
                }
                7 => {
                    if TURN {
                        b.gen_moves::<false, WQ, false, BQ, BK, Self>(self);
                    } else {
                        b.gen_moves::<true, WQ, false, BQ, BK, Self>(self);
                    }
                }
                56 => {
                    if TURN {
                        b.gen_moves::<false, WQ, WK, false, BK, Self>(self);
                    } else {
                        b.gen_moves::<true, WQ, WK, false, BK, Self>(self);
                    }
                }
                63 => {
                    if TURN {
                        b.gen_moves::<false, WQ, WK, BQ, false, Self>(self);
                    } else {
                        b.gen_moves::<true, WQ, WK, BQ, false, Self>(self);
                    }
                }
                _ => {
                    if TURN {
                        b.gen_moves::<false, WQ, WK, BQ, BK, Self>(self);
                    } else {
                        b.gen_moves::<true, WQ, WK, BQ, BK, Self>(self);
                    }
                }
            }
        }
        self.depth -= 1;
    }

    fn on_king_move<
        const TURN: bool,
        const WQ: bool,
        const WK: bool,
        const BQ: bool,
        const BK: bool,
    >(
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
            b.mov(from, to);
            if TURN {
                b.gen_moves::<false, false, false, BQ, BK, Self>(self);
            } else {
                b.gen_moves::<true, WQ, WK, false, false, Self>(self);
            }
        }
        self.depth -= 1;
    }

    fn on_ep_move<
        const TURN: bool,
        const WQ: bool,
        const WK: bool,
        const BQ: bool,
        const BK: bool,
    >(
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
            b.mov(from, to);
            if TURN {
                b.clear(to - 8);
            } else {
                b.clear(to + 8);
            }
            b.gen_moves::<false, false, false, BQ, BK, Self>(self);
        }
        self.depth -= 1;
    }

    fn on_qs_castle<
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

    fn on_pawn_push2<
        const TURN: bool,
        const WQ: bool,
        const WK: bool,
        const BQ: bool,
        const BK: bool,
    >(
        &mut self,
        me: &BitBoard,
        from: u8,
    ) {
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
