use chess::Color;
use chess::Game;
use std::io::Write;
use player::NetworkPlayer;
use player::Player;
use std::fs;
use std::fs::File;
use std::net::TcpListener;
use std::net::TcpStream;
use std::net::SocketAddr;
use std::thread;

mod player;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    println!("Server listening on port 3333");
    loop {
        let (socket, addr) = listener.accept().unwrap();
        thread::spawn(move || { handle_connection(socket, addr); });
    };
}

fn handle_connection(socket: TcpStream, addr: SocketAddr) {
    println!("Connection from {}", addr);
    let dir = format!("/var/chess-web/{}", addr.ip());
    if let Err(e) = fs::create_dir(&dir) {
        println!("Couldn't create directory {}", e);
    }

    let filename = fs::read_dir(&dir).unwrap().map(|entry| {
        let s: String = entry.unwrap().path().file_name().unwrap().to_str().unwrap().to_string();
        let len = s.len();
        let i: i32 = s[5..len-6].parse().unwrap();
        return i
    }).max().unwrap_or(0) + 1;
    let filename = "game-".to_owned() + &filename.to_string() + &".board".to_owned();
    let mut file = File::create(format!("{}/{}", dir, filename)).unwrap();

    let mut white = NetworkPlayer {
        socket,
        color: Color::White,
    };
    let mut black = white.get_opponent();
    let mut game = Game::new();
    while game.result() == None {
        file.write(format!("{}\n", game.current_position()).as_bytes()).unwrap();
        let m = match game.side_to_move() {
            Color::White => white.get_move(&game.current_position()),
            Color::Black => black.get_move(&game.current_position()),
        };
        game.make_move(m);
        if game.can_declare_draw() {
            game.declare_draw();
        }
    }
    file.write(format!("{}\n", game.current_position()).as_bytes()).unwrap();
    let result = game.result().unwrap();
    println!("{:?}", result);
    white.inform_of_result(game.current_position(), result, addr.ip().to_string(),filename);
}
