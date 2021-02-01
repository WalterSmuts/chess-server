use chess::Board;
use chess::BoardStatus;
use chess::ChessMove;
use chess::Color;
use chess::GameResult;
use chess::MoveGen;
use chess::Square;
use std::io::{Read, Write};
use std::net::TcpStream;

use rand::Rng;

const MAX_FEN_SIZE: usize = 92;
const MAX_OPPONENT_SIZE: usize = 6;

pub trait Player {
    fn get_move(&mut self, board: &Board) -> ChessMove;
}

pub struct RandomPlayer;
pub struct GreedyPlayer;
pub struct NetworkPlayer {
    pub socket: TcpStream,
    pub color: Color,
}

impl Player for RandomPlayer {
    fn get_move(&mut self, board: &Board) -> ChessMove {
        let mut moves = MoveGen::new_legal(&board);
        let mut rng = rand::thread_rng();
        let m = moves.nth(rng.gen_range(0, moves.len())).unwrap();
        return m;
    }
}

impl Player for GreedyPlayer {
    fn get_move(&mut self, board: &Board) -> ChessMove {
        let mut moves = MoveGen::new_legal(&board);
        let mut greedy_move = moves.next().unwrap();
        for m in moves.into_iter() {
            let test_board = board.make_move_new(m);
            let greedy_board = board.make_move_new(greedy_move);
            let better = match board.side_to_move() {
                Color::White => board_score(&test_board) > board_score(&greedy_board),
                Color::Black => board_score(&test_board) < board_score(&greedy_board),
            };
            if better {
                greedy_move = m;
            }
        }
        return greedy_move;
    }
}

impl Player for NetworkPlayer {
    fn get_move(&mut self, board: &Board) -> ChessMove {
        let fen = format!("{}", board);
        self.socket.write(fen.as_bytes()).unwrap();
        let mut data = [0 as u8; MAX_FEN_SIZE];
        let size = self.socket.read(&mut data).unwrap();
        let string = String::from_utf8(data[0..size].to_vec()).unwrap();
        let square_string1 = string[0..2].to_string();
        let square_string2 = string[2..4].to_string();
        ChessMove::new(
            Square::from_string(square_string1).unwrap(),
            Square::from_string(square_string2).unwrap(),
            None,
        )
    }
}

impl NetworkPlayer {
    pub fn get_opponent(&mut self) -> Box<dyn Player> {
        let mut data = [0 as u8; MAX_OPPONENT_SIZE];
        let size = self.socket.read(&mut data).unwrap();
        let opponent = String::from_utf8(data[0..size].to_vec()).unwrap();
        match opponent.as_str() {
            "Greedy" => Box::new(GreedyPlayer),
            "Random" => Box::new(RandomPlayer),
            _ => panic!("No such player exists"),
        }
    }

    pub fn inform_of_result(&mut self, board: Board, result: GameResult) -> () {
        let control_code = match result {
            GameResult::WhiteCheckmates | GameResult::BlackResigns => {
                if self.color == Color::White {
                    "W"
                } else {
                    "L"
                }
            }
            GameResult::BlackCheckmates | GameResult::WhiteResigns => {
                if self.color == Color::White {
                    "L"
                } else {
                    "W"
                }
            }
            _ => "D",
        };
        self.socket.write(control_code.as_bytes()).unwrap();
        let fen = format!("{}", board);
        self.socket.write(fen.as_bytes()).unwrap();
    }
}

fn board_score(board: &Board) -> i32 {
    if board.status() == BoardStatus::Checkmate {
        match board.side_to_move() {
            Color::White => return std::i32::MIN,
            Color::Black => return std::i32::MAX,
        }
    }
    let fen = format!("{}", board);
    let mut score: i32 = 0;
    for c in fen.chars() {
        match c {
            'p' => score = score - 1,
            'n' => score = score - 3,
            'b' => score = score - 3,
            'r' => score = score - 5,
            'q' => score = score - 9,
            'P' => score = score + 1,
            'N' => score = score + 3,
            'B' => score = score + 3,
            'R' => score = score + 5,
            'Q' => score = score + 9,
            ' ' => break,
            _ => (),
        }
    }
    score = score * 100 + MoveGen::new_legal(&board).len() as i32;
    score
}
