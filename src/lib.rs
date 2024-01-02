use board::{GameState, ChessGame};

pub mod board;
pub mod notation;
pub mod pgn;
pub mod piece;

pub fn perft(gs: &mut GameState, limit: usize) -> usize {
    if limit == 1 {
        gs.moves().len()
    }
    else {
        let moves = gs.moves();
        let mut total = 0;
        for mov in moves {
            gs.move_det(&mov);
            total += perft(gs, limit - 1);
            gs.unmove_det(&mov);
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use crate::{board::{GameState, ChessGame}, perft};
    // game boards from https://www.chessprogramming.org/Perft_Results
    #[test]
    fn perft_base() {
        let mut gs = GameState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        assert_eq!(perft(&mut gs, 1), 20);
        assert_eq!(perft(&mut gs, 2), 400);
        assert_eq!(perft(&mut gs, 3), 8902);
        assert_eq!(perft(&mut gs, 4), 197281);
        assert_eq!(perft(&mut gs, 5), 4865609);
        assert_eq!(perft(&mut gs, 6), 119060324);
    }

    #[test]
    fn perft_kiwipete() {
        let mut gs = GameState::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -").unwrap();
        assert_eq!(perft(&mut gs, 1), 48);
        assert_eq!(perft(&mut gs, 2), 2039);
        assert_eq!(perft(&mut gs, 3), 97862);
        assert_eq!(perft(&mut gs, 4), 4085603);
        assert_eq!(perft(&mut gs, 5), 193690690);
    }

    #[test]
    fn perft_pos3() {
        let mut gs = GameState::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -").unwrap();
        assert_eq!(perft(&mut gs, 1), 14);
        assert_eq!(perft(&mut gs, 2), 191);
        assert_eq!(perft(&mut gs, 3), 2812);
        assert_eq!(perft(&mut gs, 4), 43238);
        assert_eq!(perft(&mut gs, 5), 674624);
    }
}
