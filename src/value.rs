use crate::action::Action;
use crate::actions::{any_actions, is_king_attacked};
use crate::state::{Piece, State, PC, PT};

pub enum Status {
    Running,
    BlackWin,
    Tie,
    WhiteWin,
}

pub fn value(state: &State) -> Status {
    if state.drawn {
        return Status::Tie;
    }

    let actions_empty = !any_actions(state);

    if !actions_empty {
        Status::Running
    } else {
        let king_attacked = is_king_attacked(state, state.turn);

        if king_attacked {
            match state.turn.opponent() {
                PC::Black => Status::BlackWin,
                PC::White => Status::WhiteWin,
            }
        } else {
            Status::Tie
        }
    }
}

static PAWN_TABLE: &[[isize; 8]; 8] = &[
    [ 0 ,  0,  0,  0,  0,  0,  0,  0],
    [ 50, 50, 50, 50, 50, 50, 50, 50],
    [ 10, 10, 20, 30, 30, 20, 10, 10],
    [ 5 ,  5, 10, 25, 25, 10,  5,  5],
    [ 0 ,  0,  0, 20, 20,  0,  0,  0],
    [ 5 , -5,-10,  0,  0,-10, -5,  5],
    [ 5 , 10, 10,-20,-20, 10, 10,  5],
    [ 0 ,  0,  0,  0,  0,  0,  0,  0],
];
static KNIGHT_TABLE: &[[isize; 8]; 8] = &[
    [-50,-40,-30,-30,-30,-30,-40,-50],
    [-40,-20,  0,  0,  0,  0,-20,-40],
    [-30,  0, 10, 15, 15, 10,  0,-30],
    [-30,  5, 15, 20, 20, 15,  5,-30],
    [-30,  0, 15, 20, 20, 15,  0,-30],
    [-30,  5, 10, 15, 15, 10,  5,-30],
    [-40,-20,  0,  5,  5,  0,-20,-40],
    [-50,-40,-30,-30,-30,-30,-40,-50],
];
static BISHOP_TABLE: &[[isize; 8]; 8] = &[
    [ 20,-10,-10,-10,-10,-10,-10,-20],
    [ 10,  0,  0,  0,  0,  0,  0,-10],
    [ 10,  0,  5, 10, 10,  5,  0,-10],
    [ 10,  5,  5, 10, 10,  5,  5,-10],
    [ 10,  0, 10, 10, 10, 10,  0,-10],
    [ 10, 10, 10, 10, 10, 10, 10,-10],
    [ 10,  5,  0,  0,  0,  0,  5,-10],
    [ 20,-10,-10,-10,-10,-10,-10,-20],
];
static ROOK_TABLE: &[[isize; 8]; 8] = &[
    [  0,  0,  0,  0,  0,  0,  0,  0],
    [  5, 10, 10, 10, 10, 10, 10,  5],
    [ -5,  0,  0,  0,  0,  0,  0, -5],
    [ -5,  0,  0,  0,  0,  0,  0, -5],
    [ -5,  0,  0,  0,  0,  0,  0, -5],
    [ -5,  0,  0,  0,  0,  0,  0, -5],
    [ -5,  0,  0,  0,  0,  0,  0, -5],
    [  0,  0,  0,  5,  5,  0,  0,  0],
];
static QUEEN_TABLE: &[[isize; 8]; 8] = &[
    [-20,-10,-10, -5, -5,-10,-10,-20],
    [-10,  0,  0,  0,  0,  0,  0,-10],
    [-10,  0,  5,  5,  5,  5,  0,-10],
    [ -5,  0,  5,  5,  5,  5,  0, -5],
    [  0,  0,  5,  5,  5,  5,  0, -5],
    [-10,  5,  5,  5,  5,  5,  0,-10],
    [-10,  0,  5,  0,  0,  0,  0,-10],
    [-20,-10,-10, -5, -5,-10,-10,-20],
];
static KING_TABLE: &[[isize; 8]; 8] = &[
    [-30,-40,-40,-50,-50,-40,-40,-30],
    [-30,-40,-40,-50,-50,-40,-40,-30],
    [-30,-40,-40,-50,-50,-40,-40,-30],
    [-30,-40,-40,-50,-50,-40,-40,-30],
    [-20,-30,-30,-40,-40,-30,-30,-20],
    [-10,-20,-20,-20,-20,-20,-20,-10],
    [ 20, 20,  0,  0,  0,  0, 20, 20],
    [ 20, 30, 10,  0,  0, 10, 30, 20],
];

const PIECE_WORTH_COEFFICIENT: f64 = 100.0;
fn piece_worth(pt: PT) -> f64 {
    match pt {
        PT::Pawn { .. } => 1.0,
        PT::Knight => 3.2,
        PT::Bishop => 3.33,
        PT::Rook { .. } => 5.1,
        PT::Queen => 8.8,
        PT::King { .. } => 100000.0,
    }
}

fn piece_position_worth(pc: PC, pt: PT, i: usize, j: usize) -> isize {
    let table = match pt {
        PT::Pawn { .. } => PAWN_TABLE,
        PT::Knight => KNIGHT_TABLE,
        PT::Bishop => BISHOP_TABLE,
        PT::Rook { .. } => ROOK_TABLE,
        PT::Queen => QUEEN_TABLE,
        PT::King { .. } => KING_TABLE,
    };
    match pc {
        PC::Black => table[7 - i][j],
        PC::White => table[i][j],
    }
}

fn piece_value(p: &Piece, i: usize, j: usize) -> f64 {
    PIECE_WORTH_COEFFICIENT * piece_worth(p.t) + piece_position_worth(p.c, p.t, i, j) as f64
}

pub fn heuristic(state: &State) -> f64 {
    let mut value = 0.0;

    for i in 0..8 {
        for j in 0..8 {
            if let Some(p) = state.board[i][j] {
                if p.c == state.turn {
                    value += piece_value(&p, i, j);
                } else {
                    value -= piece_value(&p, i, j);
                }
            };
        }
    }

    value
}

pub fn heuristic_action(state: &State, action: &Action) -> f64 {
    if *action == Action::Tie {
        return 0.;
    }

    let main_piece = action
        .get_main_piece(state)
        .expect("Invalid action found in heuristic_action");

    match *action {
        Action::Jump { s_y, s_x, e_y, e_x } => {
            piece_value(&main_piece, e_y, e_x) - piece_value(&main_piece, s_y, s_x)
        }
        Action::Capture { s_y, s_x, e_y, e_x } => {
            let captured_piece = state.board[e_y][e_x].expect("Invalid capture action found");
            piece_value(&main_piece, e_y, e_x) - piece_value(&main_piece, s_y, s_x)
                + piece_value(&captured_piece, e_y, e_x)
        }
        Action::Castling {
            s_y,
            s_x: _,
            queenside,
        } => {
            if queenside {
                let rook = state.board[s_y][0].expect("Invalid castling action found");
                piece_value(&main_piece, s_y, 2) - piece_value(&main_piece, s_y, 4)
                    + piece_value(&rook, s_y, 3)
                    - piece_value(&rook, s_y, 0)
            } else {
                let rook = state.board[s_y][7].expect("Invalid castling action found");
                piece_value(&main_piece, s_y, 6) - piece_value(&main_piece, s_y, 4)
                    + piece_value(&rook, s_y, 5)
                    - piece_value(&rook, s_y, 7)
            }
        }
        Action::Promotion {
            s_y,
            s_x,
            e_y,
            e_x,
            to_piece,
        } => {
            let captured_value = match state.board[e_y][e_x] {
                None => 0.,
                Some(captured_piece) => piece_value(&captured_piece, e_y, e_x),
            };

            let new_piece = Piece {
                c: main_piece.c,
                t: to_piece,
            };
            piece_value(&new_piece, e_y, e_x) - piece_value(&main_piece, s_y, s_x) + captured_value
        }
        Action::Enpassant { s_y, s_x, e_y, e_x } => {
            let captured_pawn = state.board[s_y][e_x].expect("Invalid enpassant action found");
            piece_value(&main_piece, e_y, e_x) - piece_value(&main_piece, s_y, s_x)
                + piece_value(&captured_pawn, s_y, e_x)
        }
        Action::Tie => 0.,
    }
}
