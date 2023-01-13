use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum PC {
    Black,
    White,
}

impl PC {
    pub fn opponent(&self) -> PC {
        match self {
            PC::Black => PC::White,
            PC::White => PC::Black,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PT {
    Pawn { last_jump: Option<usize> },
    Knight,
    Bishop,
    Rook { has_moved: bool },
    Queen,
    King { has_moved: bool },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Piece {
    pub c: PC,
    pub t: PT,
}

impl Piece {
    pub fn piece_to_char(&self) -> char {
        match self {
            Piece {
                c: PC::Black,
                t: PT::Pawn { .. },
            } => '♟',
            Piece {
                c: PC::Black,
                t: PT::Knight,
            } => '♞',
            Piece {
                c: PC::Black,
                t: PT::Bishop,
            } => '♝',
            Piece {
                c: PC::Black,
                t: PT::Rook { .. },
            } => '♜',
            Piece {
                c: PC::Black,
                t: PT::Queen,
            } => '♛',
            Piece {
                c: PC::Black,
                t: PT::King { .. },
            } => '♚',
            Piece {
                c: PC::White,
                t: PT::Pawn { .. },
            } => '♙',
            Piece {
                c: PC::White,
                t: PT::Knight,
            } => '♘',
            Piece {
                c: PC::White,
                t: PT::Bishop,
            } => '♗',
            Piece {
                c: PC::White,
                t: PT::Rook { .. },
            } => '♖',
            Piece {
                c: PC::White,
                t: PT::Queen,
            } => '♕',
            Piece {
                c: PC::White,
                t: PT::King { .. },
            } => '♔',
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub board: [[Option<Piece>; 8]; 8],
    pub hash_to_occurences: HashMap<u64, i32>,
    pub turn: PC,
    pub moves: usize,
    pub drawn: bool,
}

impl State {
    pub fn hash(&self) -> u64 {
        const B: u64 = 1156366624;
        const MOD: u64 = 999999999999989;
        const LOOKUP: [[u64; 8]; 8] = [
            [37, 1369, 50653, 1874161, 69343957, 2565726409, 94931877133, 3512479453921],
            [129961739795077, 808584372417893, 917621779462360, 952005840107683, 224216083984656, 295995107432360, 951818974997430, 217302074905295],
            [40176771496003, 486540545352122, 2000178028712, 74006587062344, 738243721306750, 315017688350047, 655654468951860, 259215351219084],
            [590967995106207, 865815818929890, 35185300406282, 301856115032445, 168676256200586, 241021479421748, 917794738604764, 958405328376631],
            [460997149935732, 56894547622271, 105098262024049, 888635694889846, 879520710924654, 542266304212550, 63853255864570, 362570466989112],
            [415107278597287, 358969308099784, 281864399692151, 428982788609697, 872363178558954, 277437606681650, 265191447221160, 812083547183019],
            [47091245772033, 742376093565232, 467915461913881, 312872090813784, 576267360110129, 321892324075004, 910015990775269, 670591658685316],
            [811891371356956, 39980740207702, 479287387684985, 733633344344632, 144433740751681, 344048407812252, 729791089053456, 2270294978169],
        ];

        let mut v = B;
        v += match self.turn {
            PC::Black => 17,
            PC::White => 18,
        };
        v %= MOD;

        for (i, board_row) in self.board.iter().enumerate() {
            for (j, c) in board_row.iter().enumerate() {
                let piece_hash = match c {
                    None => 1,
                    Some(p) => {
                        let color_hash = match p.c {
                            PC::Black => 0,
                            PC::White => 1,
                        };

                        let type_hash = match p.t {
                            PT::Pawn { last_jump }
                                if self.moves > 0 && last_jump == Some(self.moves - 1) =>
                            {
                                0
                            }
                            PT::Pawn { .. } => 1,
                            PT::Knight => 2,
                            PT::Bishop => 3,
                            PT::Rook { has_moved: false } => 4,
                            PT::Rook { has_moved: true } => 5,
                            PT::Queen => 6,
                            PT::King { has_moved: false } => 7,
                            PT::King { has_moved: true } => 8,
                        };

                        2 + 2 * type_hash + color_hash
                    }
                };

                v += LOOKUP[i][j] * piece_hash;
                v %= MOD;
            }
        }

        v
    }

    pub fn new() -> Self {
        let initial = [
            "♜♞♝♛♚♝♞♜",
            "♟♟♟♟♟♟♟♟",
            "        ",
            "        ",
            "        ",
            "        ",
            "♙♙♙♙♙♙♙♙",
            "♖♘♗♕♔♗♘♖",
        ];

        let map_initial_to_piece = |c: char| match c {
            '♟' => Some(Piece {
                c: PC::Black,
                t: PT::Pawn { last_jump: None },
            }),
            '♞' => Some(Piece {
                c: PC::Black,
                t: PT::Knight,
            }),
            '♝' => Some(Piece {
                c: PC::Black,
                t: PT::Bishop,
            }),
            '♜' => Some(Piece {
                c: PC::Black,
                t: PT::Rook { has_moved: false },
            }),
            '♛' => Some(Piece {
                c: PC::Black,
                t: PT::Queen,
            }),
            '♚' => Some(Piece {
                c: PC::Black,
                t: PT::King { has_moved: false },
            }),
            '♙' => Some(Piece {
                c: PC::White,
                t: PT::Pawn { last_jump: None },
            }),
            '♘' => Some(Piece {
                c: PC::White,
                t: PT::Knight,
            }),
            '♗' => Some(Piece {
                c: PC::White,
                t: PT::Bishop,
            }),
            '♖' => Some(Piece {
                c: PC::White,
                t: PT::Rook { has_moved: false },
            }),
            '♕' => Some(Piece {
                c: PC::White,
                t: PT::Queen,
            }),
            '♔' => Some(Piece {
                c: PC::White,
                t: PT::King { has_moved: false },
            }),
            ' ' => None,
            _ => panic!("Unexpected character found: {}", c),
        };

        let mut board = [[None; 8]; 8];

        for (i, initial_row) in initial.iter().enumerate() {
            for (j, c) in initial_row.chars().enumerate() {
                board[i][j] = map_initial_to_piece(c);
            }
        }

        let mut state = State {
            board,
            hash_to_occurences: HashMap::new(),
            turn: PC::White,
            moves: 0,
            drawn: false,
        };

        *state.hash_to_occurences.entry(state.hash()).or_insert(0) += 1;

        state
    }
    pub fn board_to_string(&self) -> String {
        let mut hello = String::new();
        for i in self.board.iter() {
            for j in i {
                let letter = match j {
                    None => ' ',
                    Some(p) => p.piece_to_char(),
                };
                hello.push(letter);
                hello.push(' ');
            }
            hello.push('\n');
        }
        hello
    }
}

impl Default for State {
    fn default() -> Self {
        State::new()
    }
}
