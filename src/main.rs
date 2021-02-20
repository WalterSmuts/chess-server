use chess::Color;
use lazy_static::lazy_static;
use player::GreedyPlayer;
use player::NetworkPlayer;
use player::Player;
use player::RandomPlayer;
use std::fs;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::Mutex;
use std::thread;

mod game_coordinator;
mod player;

lazy_static! {
    static ref WAITING_PLAYER: Mutex<Option<Box<NetworkPlayer>>> = Mutex::new(None);
}

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

    let player2: Box<dyn Player> = match player1.get_opponent().as_str() {
        "Greedy" => Box::new(GreedyPlayer),
        "Random" => Box::new(RandomPlayer),
        "Network" => {
            let mut mutex_guard = WAITING_PLAYER.lock().unwrap();
            let option = mutex_guard.take();
            match option {
                Some(player) => {
                    if !(*player).alive() {
                        *mutex_guard = Some(player1);
                        return
                    }
                    player
                },
                None => {
                    *mutex_guard = Some(player1);
                    return
                }
            }
        }
        _ => panic!("No such player exists"),
    };

    let mut game_coordinator = game_coordinator::GameCoordinator::new(player1, player2, &dir);
    game_coordinator.run();
}
