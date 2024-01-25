use fes::{
    bit_board::{print_bitmask, BitBoardGame},
    game::ChessGame,
};

fn main() {
    let mut gs = BitBoardGame::from_fen("8/8/2R5/5r2/4PN2/1k6/2p2K2/8 w - - 0 1").unwrap();

    println!();
    println!("{}", gs.board);

    gs.moves();
}
