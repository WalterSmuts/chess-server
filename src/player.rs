use chess::Board;
use chess::BoardStatus;
use chess::ChessMove;
use chess::Color;
use chess::GameResult;
use chess::MoveGen;
use chess::Square;
use std::io::ErrorKind;
use std::io::{Read, Write};
use std::net::TcpStream;

use rand::Rng;

pub trait Player {
    fn get_move(&mut self, board: &Board) -> ChessMove;
    fn inform_of_result(&mut self, board: Board, result: GameResult, filename: &String) {
        println!(
            "Final board state {} result {:?} at {}",
            board, result, filename
        );
    }
}

impl std::fmt::Debug for dyn Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LocalPlayer")
    }
}

pub struct RandomPlayer;
pub struct GreedyPlayer;
pub struct NetworkPlayer {
    pub socket: TcpStream,
    pub color: Color,
}

impl Player for RandomPlayer {
    fn get_move(&mut self, board: &Board) -> ChessMove {
        let mut moves = MoveGen::new_legal(board);
        let mut rng = rand::thread_rng();

        moves.nth(rng.gen_range(0, moves.len())).unwrap()
    }
}

impl Player for GreedyPlayer {
    fn get_move(&mut self, board: &Board) -> ChessMove {
        let mut moves = MoveGen::new_legal(board);
        let mut greedy_move = moves.next().unwrap();
        for m in moves {
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
        greedy_move
    }
}

impl Player for NetworkPlayer {
    fn get_move(&mut self, board: &Board) -> ChessMove {
        let fen = format!("{}", board);
        write_lenth_prefixed(&mut self.socket, fen.as_bytes()).unwrap();
        let data = read_lenth_prefixed(&mut self.socket);
        let string = String::from_utf8(data).unwrap();
        let square_string1 = string[0..2].to_string();
        let square_string2 = string[2..4].to_string();
        ChessMove::new(
            Square::from_string(square_string1).unwrap(),
            Square::from_string(square_string2).unwrap(),
            None,
        )
    }

    fn inform_of_result(&mut self, board: Board, result: GameResult, filename: &String) {
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

        write_lenth_prefixed(&mut self.socket, control_code.as_bytes()).unwrap();
        let fen = format!("{}", board);
        write_lenth_prefixed(&mut self.socket, fen.as_bytes()).unwrap();
        write_lenth_prefixed(
            &mut self.socket,
            format!("https://chess.waltersmuts.com/{}.html", filename).as_bytes(),
        )
        .unwrap();
    }
}

impl NetworkPlayer {
    pub fn get_opponent(&mut self) -> String {
        let data = read_lenth_prefixed(&mut self.socket);
        String::from_utf8(data).unwrap()
    }

    pub fn alive(&self) -> bool {
        let mut buf = [0; 0];
        self.socket.set_nonblocking(true).unwrap();
        let result = self.socket.peek(&mut buf);
        self.socket.set_nonblocking(false).unwrap();
        // Strangely enough, if the client dropped the connection we get a Ok here and if the
        // client haven't responded yet, but is still connected, then we get an `WouldBlock`
        match result {
            Err(e) => match e.kind() {
                ErrorKind::WouldBlock => true,
                _ => false,
            },
            Ok(_) => false,
        }
    }
}

fn write_lenth_prefixed(socket: &mut TcpStream, buf: &[u8]) -> std::io::Result<usize> {
    let len = vec![buf.len() as u8];
    socket.write(&len)?;
    socket.write(buf)
}

fn read_lenth_prefixed(socket: &mut TcpStream) -> Vec<u8> {
    let mut len = vec![0u8; 1];
    socket.read(&mut len).unwrap();
    let mut buf = [0_u8; u8::MAX as usize];
    socket.read(&mut buf).unwrap();
    buf[0..len[0] as usize].to_vec()
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
            'p' => score -= 1,
            'n' => score -= 3,
            'b' => score -= 3,
            'r' => score -= 5,
            'q' => score -= 9,
            'P' => score += 1,
            'N' => score += 3,
            'B' => score += 3,
            'R' => score += 5,
            'Q' => score += 9,
            ' ' => break,
            _ => (),
        }
    }
    score = score * 100 + MoveGen::new_legal(board).len() as i32;
    score
}
