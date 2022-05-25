use crate::player::Player;
use chess::Color;
use chess::Game;
use std::fs;
use std::fs::File;
use std::io::Write;

pub struct GameCoordinator {
    player1: Box<dyn Player>,
    player2: Box<dyn Player>,
    file: File,
    filename: String,
    game: Game,
}

impl GameCoordinator {
    pub fn new(player1: Box<dyn Player>, player2: Box<dyn Player>, ip: &String) -> GameCoordinator {
        let dir = format!("/var/chess-web/{}", ip);
        if let Err(e) = fs::create_dir(&dir) {
            println!("Couldn't create directory {}", e);
        }

        let filename = std::fs::read_dir(&dir)
            .unwrap()
            .map(|entry| {
                let s: String = entry
                    .unwrap()
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                let len = s.len();
                let i: i32 = s[5..len - 6].parse().unwrap();
                i
            })
            .max()
            .unwrap_or(0)
            + 1;
        let filename = "game-".to_owned() + &filename.to_string() + ".board";
        let file = File::create(format!("{}/{}", dir, filename)).unwrap();
        let filename = format!("{}/{}", ip, filename);

        GameCoordinator {
            game: Game::new(),
            player1,
            player2,
            filename,
            file,
        }
    }

    pub fn run(&mut self) {
        let mut file = &self.file;

        while self.game.result() == None {
            file.write_all(format!("{}\n", self.game.current_position()).as_bytes())
                .unwrap();
            let m = match self.game.side_to_move() {
                Color::White => self.player1.get_move(&self.game.current_position()),
                Color::Black => self.player2.get_move(&self.game.current_position()),
            };
            self.game.make_move(m);
            if self.game.can_declare_draw() {
                self.game.declare_draw();
            }
        }
        file.write_all(format!("{}\n", self.game.current_position()).as_bytes())
            .unwrap();
        let result = self.game.result().unwrap();
        println!("{:?}", result);
        self.player1
            .inform_of_result(self.game.current_position(), result, &self.filename);
        self.player2
            .inform_of_result(self.game.current_position(), result, &self.filename);
    }
}
