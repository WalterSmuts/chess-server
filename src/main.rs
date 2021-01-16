use chess::Board;
use chess::Color;
use chess::Game;
use player::NetworkPlayer;
use player::Player;
use std::net::TcpListener;

mod player;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    println!("Server listening on port 3333");
    let (sock1, _) = listener.accept().unwrap();
    let mut white = NetworkPlayer {
        socket: sock1,
        color: Color::White,
    };
    let mut black = white.get_opponent();
    let mut game = Game::new();
    while game.result() == None {
        print_board(&game.current_position());
        let m = match game.side_to_move() {
            Color::White => white.get_move(&game.current_position()),
            Color::Black => black.get_move(&game.current_position()),
        };
        game.make_move(m);
        if game.can_declare_draw() {
            game.declare_draw();
        }
    }
    print_board(&game.current_position());
    let result = game.result().unwrap();
    println!("{:?}", result);
    white.inform_of_result(result);
    drop(listener);
}

pub fn print_board(board: &Board) {
    println!("    a   b   c   d   e   f   g   h");
    let fen = format!("{}", board);

    println!("  {}", "-".repeat(33));
    let mut i = 0;
    print!("{} ", 8 - i);
    print!("|");
    for c in fen.chars() {
        if c.is_numeric() {
            print!("{}", "   |".repeat(c.to_digit(10).unwrap() as usize));
        } else if c == '/' {
            print!(" {}", 8 - i);
            i = i + 1;
            print!("\n");
            println!("  {}", "-".repeat(33));
            print!("{} ", 8 - i);
            print!("|");
        } else if c != ' ' {
            print!(" {} |", c);
        } else {
            break;
        }
    }
    print!(" {}", 8 - i);
    print!("\n");
    println!("  {}", "-".repeat(33));
    println!("    a   b   c   d   e   f   g   h");
    println!("{:?}'s turn to move.\n", board.side_to_move());
}
