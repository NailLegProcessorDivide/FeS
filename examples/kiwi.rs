use fes::{perft, board::GameState, game::ChessGame};


fn main() {
    let mut gs = GameState::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -").unwrap();
    assert_eq!(perft(&mut gs, 1), 48);
    assert_eq!(perft(&mut gs, 2), 2039);
    assert_eq!(perft(&mut gs, 3), 97862);
    assert_eq!(perft(&mut gs, 4), 4085603);
    assert_eq!(perft(&mut gs, 5), 193690690);
}