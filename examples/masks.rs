use fes::{
    bit_board::{print_bitmask, BitBoardGame},
    game::ChessGame,
};

fn main() {
    let mut gs = BitBoardGame::from_fen("8/2p5/3p4/KP5r/1R2Pp1k/8/6P1/8 b - - 0 1").unwrap();

    println!();

    gs.board.set(63, 0b1011);

    gs.board.set(8, 0b0011);


    print!("{}", gs.board);

    //print_bitmask(gs.board.hor_pin_mask2::<false>() & (0xff << (8 * 3)));



    println!("\n\n");

    gs.moves();
}
