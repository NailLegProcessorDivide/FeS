use crate::bit_board::{OnMove, BitBoard, BoolExists};



struct PerftMove {
    depth_target: u64,
    depth: u64,
    counter: u64,
}

impl OnMove for PerftMove {
    fn on_move<const TURN: bool, const EP: bool, const WQ: bool,
        const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard, from: u8, to: u8)
        where BoolExists<{!TURN}>: Sized, {
        self.depth += 1;
        if self.depth == self.depth_target {
            self.counter += 1;
        }
        else {
            let mut b = me.clone();
            b.dupe(from, to);
            b.clear(from);
            if TURN {
                b.gen_moves::<false, false, WQ, WK, BQ, BK, Self>(self);
            }
            else {
                b.gen_moves::<true, false, WQ, WK, BQ, BK, Self>(self);
            }
        }
        self.depth -= 1;
    }

    fn on_ep_move<const TURN: bool, const EP: bool, const WQ: bool,
        const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard, from: u8, to: u8) {
        todo!()
    }

    fn on_qs_castle<const TURN: bool, const EP: bool, const WQ: bool,
        const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard, from: u8, to: u8) {
        todo!()
    }

    fn on_ks_castle<const TURN: bool, const EP: bool, const WQ: bool,
        const WK: bool, const BQ: bool, const BK: bool>(&mut self, me: &BitBoard, from: u8, to: u8) {
        todo!()
    }
}
