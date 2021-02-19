use chess::Color;
use player::NetworkPlayer;
use std::fs;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;

mod game_coordinator;
mod player;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    println!("Server listening on port 3333");
    loop {
        let (socket, addr) = listener.accept().unwrap();
        thread::spawn(move || {
            handle_connection(socket, addr);
        });
    }
}

fn handle_connection(socket: TcpStream, addr: SocketAddr) {
    println!("Connection from {}", addr);
    let dir = format!("/var/chess-web/{}", addr.ip());
    if let Err(e) = fs::create_dir(&dir) {
        println!("Couldn't create directory {}", e);
    }

    let mut player1 = Box::new(NetworkPlayer {
        addr,
        socket,
        color: Color::White,
    });

    let player2 = player1.get_opponent();

    let mut game_coordinator = game_coordinator::GameCoordinator::new(player1, player2, &dir);
    game_coordinator.run();
}
