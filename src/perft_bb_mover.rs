use crate::bit_board::{OnMove, BitBoard, BoolExists};



struct PerftMove {
    depth_target: u64,
    depth: u64,
    counter: u64,
}

impl OnMove for PerftMove {
    fn on_move<const TURN: bool, const EP: bool, const WQ: bool,
        const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard, from: u8, to: u8) {
        self.depth += 1;
        if self.depth == self.depth_target {
            self.counter += 1;
        }
        else {
            let mut b = me.clone();
            b.mov(from, to);
            if TURN {
                b.gen_moves::<false, false, WQ, WK, BQ, BK, Self>(self);
            }
            else {
                b.gen_moves::<true, false, WQ, WK, BQ, BK, Self>(self);
            }
        }
        self.depth -= 1;
    }

    fn on_rook_move<const TURN: bool, const EP: bool, const WQ: bool,
        const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard, from: u8, to: u8) {
            self.depth += 1;
            if self.depth == self.depth_target {
                self.counter += 1;
            }
            else {
                let mut b = me.clone();
                b.mov(from, to);
                match from {
                    0 => {
                        if TURN {
                            b.gen_moves::<false, false, false, WK, BQ, BK, Self>(self);
                        }
                        else {
                            b.gen_moves::<true, false, false, WK, BQ, BK, Self>(self);
                        }
                    }
                    7 => {
                        if TURN {
                            b.gen_moves::<false, false, WQ, false, BQ, BK, Self>(self);
                        }
                        else {
                            b.gen_moves::<true, false, WQ, false, BQ, BK, Self>(self);
                        }
                    }
                    56 => {
                        if TURN {
                            b.gen_moves::<false, false, WQ, WK, false, BK, Self>(self);
                        }
                        else {
                            b.gen_moves::<true, false, WQ, WK, false, BK, Self>(self);
                        }
                    }
                    63 => {
                        if TURN {
                            b.gen_moves::<false, false, WQ, WK, BQ, false, Self>(self);
                        }
                        else {
                            b.gen_moves::<true, false, WQ, WK, BQ, false, Self>(self);
                        }
                    }
                    _ => {
                        if TURN {
                            b.gen_moves::<false, false, WQ, WK, BQ, BK, Self>(self);
                        }
                        else {
                            b.gen_moves::<true, false, WQ, WK, BQ, BK, Self>(self);
                        }
                    }
                }
            }
            self.depth -= 1;
    }

    fn on_king_move<const TURN: bool, const EP: bool, const WQ: bool,
        const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard, from: u8, to: u8) {
        self.depth += 1;
        if self.depth == self.depth_target {
            self.counter += 1;
        }
        else {
            let mut b = me.clone();
            b.mov(from, to);
            if TURN {
                b.gen_moves::<false, false, false, false, BQ, BK, Self>(self);
            }
            else {
                b.gen_moves::<true, false, WQ, WK, false, false, Self>(self);
            }
        }
        self.depth -= 1;
    }

    fn on_ep_move<const TURN: bool, const EP: bool, const WQ: bool,
        const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard, from: u8, to: u8) {
        todo!()
    }

    fn on_qs_castle<const TURN: bool, const EP: bool, const WQ: bool,
        const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard) {
            self.depth += 1;
            if self.depth == self.depth_target {
                self.counter += 1;
            }
            else {
                let mut b = me.clone();
                if TURN {
                    b.mov(0, 3);
                    b.mov(4, 2);
                    b.gen_moves::<false, false, false, false, BQ, BK, Self>(self);
                }
                else {
                    b.mov(56, 59);
                    b.mov(60, 58);
                    b.gen_moves::<true, false, WQ, WK, false, false, Self>(self);
                }
            }
    }

    fn on_ks_castle<const TURN: bool, const EP: bool, const WQ: bool,
        const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard) {
            self.depth += 1;
            if self.depth == self.depth_target {
                self.counter += 1;
            }
            else {
                let mut b = me.clone();
                if TURN {
                    b.mov(7, 5);
                    b.mov(4, 6);
                    b.gen_moves::<false, false, false, false, BQ, BK, Self>(self);
                }
                else {
                    b.mov(63, 61);
                    b.mov(60, 62);
                    b.gen_moves::<true, false, WQ, WK, false, false, Self>(self);
                }
            }
    }

    fn on_pawn_push2<const TURN: bool, const EP: bool, const WQ: bool,
        const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard) {
        todo!()
    }
}
