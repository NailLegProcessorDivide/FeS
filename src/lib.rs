use board::{GameState, ChessGame};

pub mod board;
pub mod notation;
pub mod pgn;
pub mod piece;

pub fn perft(gs: &mut GameState, limit: usize) -> usize {
    if limit == 0 {
        1
    }
    else if limit == 1 {
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

pub fn perft_div(gs: &mut GameState, limit: usize) -> usize {
    let mut total = 0;
    for mov in gs.moves().iter() {
        gs.move_det(mov);
        let c = perft(gs, limit - 1);
        total += c;
        let ox = ('a' as u8 + (mov.from & 7)) as char;
        let oy = ('1' as u8 + (mov.from >> 3)) as char;
        let nx = ('a' as u8 + (mov.to & 7)) as char;
        let ny = ('1' as u8 + (mov.to >> 3)) as char;
        println!("{}{}{}{}: {}", ox, oy, nx, ny, c);
        gs.unmove_det(mov);
    }
    println!("total: {total}");
    total
}

#[cfg(test)]
mod tests {
    use crate::{board::{GameState, ChessGame}, perft, perft_div};
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
        assert_eq!(perft(&mut gs, 6), 11030083);
        assert_eq!(perft(&mut gs, 7), 178633661);
    }

    #[test]
    fn perft_pos4() {
        let mut gs = GameState::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1").unwrap();
        assert_eq!(perft(&mut gs, 1), 6);
        assert_eq!(perft(&mut gs, 2), 264);
        assert_eq!(perft(&mut gs, 3), 9467);
        assert_eq!(perft(&mut gs, 4), 422333);
        assert_eq!(perft(&mut gs, 5), 15833292);
        assert_eq!(perft(&mut gs, 6), 706045033);
    }

    #[test]
    fn perft_pos4b() {
        let mut gs = GameState::from_fen("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1").unwrap();
        assert_eq!(perft(&mut gs, 1), 6);
        assert_eq!(perft(&mut gs, 2), 264);
        assert_eq!(perft(&mut gs, 3), 9467);
        assert_eq!(perft(&mut gs, 4), 422333);
        assert_eq!(perft(&mut gs, 5), 15833292);
        assert_eq!(perft(&mut gs, 6), 706045033);
    }

    #[test]
    fn perft_pos5() {
        let mut gs = GameState::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();
        assert_eq!(perft(&mut gs, 1), 44);
        assert_eq!(perft(&mut gs, 2), 1486);
        assert_eq!(perft(&mut gs, 3), 62379);
        assert_eq!(perft(&mut gs, 4), 2103487);
        assert_eq!(perft(&mut gs, 5), 89941194);
    }

    #[test]
    fn perft_pos6() {
        let mut gs = GameState::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10").unwrap();
        assert_eq!(perft(&mut gs, 1), 46);
        assert_eq!(perft(&mut gs, 2), 2079);
        assert_eq!(perft(&mut gs, 3), 89890);
        assert_eq!(perft(&mut gs, 4), 3894594);
        assert_eq!(perft(&mut gs, 5), 164075551);
    }
}
