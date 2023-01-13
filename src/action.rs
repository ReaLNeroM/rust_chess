use crate::state::{Piece, State, PT};

#[derive(Clone, PartialEq, Eq, Copy, Debug)]
pub enum Action {
    Jump {
        s_y: usize,
        s_x: usize,
        e_y: usize,
        e_x: usize,
    },
    Capture {
        s_y: usize,
        s_x: usize,
        e_y: usize,
        e_x: usize,
    },
    Castling {
        s_y: usize,
        s_x: usize,
        queenside: bool,
    },
    Promotion {
        s_y: usize,
        s_x: usize,
        e_y: usize,
        e_x: usize,
        to_piece: PT,
    },
    Enpassant {
        s_y: usize,
        s_x: usize,
        e_y: usize,
        e_x: usize,
    },
    Tie,
}

fn to_row(i: usize) -> usize {
    8 - i
}
fn to_column(i: usize) -> char {
    (97 + i as u8) as char
}

impl Action {
    pub fn get_main_coords(&self) -> (usize, usize) {
        match self {
            Action::Jump { s_y, s_x, .. } => (*s_y, *s_x),
            Action::Capture { s_y, s_x, .. } => (*s_y, *s_x),
            Action::Castling { s_y, s_x, .. } => (*s_y, *s_x),
            Action::Promotion { s_y, s_x, .. } => (*s_y, *s_x),
            Action::Enpassant { s_y, s_x, .. } => (*s_y, *s_x),
            Action::Tie => (0, 0), // hack
        }
    }
    pub fn get_end_coords(&self) -> (usize, usize) {
        match self {
            Action::Jump { e_y, e_x, .. } => (*e_y, *e_x),
            Action::Capture { e_y, e_x, .. } => (*e_y, *e_x),
            Action::Castling {
                s_y,
                queenside: true,
                ..
            } => (*s_y, 2),
            Action::Castling {
                s_y,
                queenside: false,
                ..
            } => (*s_y, 6),
            Action::Promotion { e_y, e_x, .. } => (*e_y, *e_x),
            Action::Enpassant { e_y, e_x, .. } => (*e_y, *e_x),
            Action::Tie => (0, 0), // hack
        }
    }
    pub fn from_context_and_coords(
        state: &State,
        start_coords: (usize, usize),
        end_coords: (usize, usize),
        promotion_piece: Option<PT>,
    ) -> Option<Action> {
        match state.board[start_coords.0][start_coords.1] {
            Some(main_piece) => {
                if main_piece.c != state.turn {
                    return None;
                }
                if let Some(pt) = promotion_piece {
                    // must be promotion
                    return Some(Action::Promotion {
                        s_y: start_coords.0,
                        s_x: start_coords.1,
                        e_y: end_coords.0,
                        e_x: end_coords.1,
                        to_piece: pt,
                    });
                }
                if state.board[end_coords.0][end_coords.1].is_some() {
                    // must be capture
                    return Some(Action::Capture {
                        s_y: start_coords.0,
                        s_x: start_coords.1,
                        e_y: end_coords.0,
                        e_x: end_coords.1,
                    });
                }
                match main_piece.t {
                    PT::King { .. } => {
                        // could be jump, or castling
                        if start_coords.0 == end_coords.0
                            && start_coords.1 == 4
                            && end_coords.1 == 2
                        {
                            return Some(Action::Castling {
                                s_y: start_coords.0,
                                s_x: start_coords.1,
                                queenside: true,
                            });
                        } else if start_coords.0 == end_coords.0
                            && start_coords.1 == 4
                            && end_coords.1 == 6
                        {
                            return Some(Action::Castling {
                                s_y: start_coords.0,
                                s_x: start_coords.1,
                                queenside: false,
                            });
                        }

                        Some(Action::Jump {
                            s_y: start_coords.0,
                            s_x: start_coords.1,
                            e_y: end_coords.0,
                            e_x: end_coords.1,
                        })
                    }
                    PT::Pawn { .. } => {
                        // could be jump or en-passant
                        if end_coords.1 != start_coords.1 {
                            return Some(Action::Enpassant {
                                s_y: start_coords.0,
                                s_x: start_coords.1,
                                e_y: end_coords.0,
                                e_x: end_coords.1,
                            });
                        }
                        Some(Action::Jump {
                            s_y: start_coords.0,
                            s_x: start_coords.1,
                            e_y: end_coords.0,
                            e_x: end_coords.1,
                        })
                    }
                    _ => Some(Action::Jump {
                        s_y: start_coords.0,
                        s_x: start_coords.1,
                        e_y: end_coords.0,
                        e_x: end_coords.1,
                    }),
                }
            }
            None => None,
        }
    }
    pub fn get_main_piece(&self, state: &State) -> Option<Piece> {
        let coords = self.get_main_coords();
        state.board[coords.0][coords.1]
    }
    pub fn to_string(&self, state: &State) -> String {
        if *self == Action::Tie {
            return "½–½".to_string();
        }

        let piece_char = self.get_main_piece(state).unwrap().piece_to_char();

        match *self {
            Action::Jump { s_y, s_x, e_y, e_x } => {
                let s_row = to_row(s_y);
                let s_column = to_column(s_x);
                let e_row = to_row(e_y);
                let e_column = to_column(e_x);

                format!("{} {}{}{}{}", piece_char, s_column, s_row, e_column, e_row)
            }
            Action::Capture { s_y, s_x, e_y, e_x } => {
                let s_row = to_row(s_y);
                let s_column = to_column(s_x);
                let e_row = to_row(e_y);
                let e_column = to_column(e_x);

                format!("{} {}{}x{}{}", piece_char, s_column, s_row, e_column, e_row)
            }
            Action::Castling { queenside, .. } => {
                if queenside {
                    "0-0-0".to_string()
                } else {
                    "0-0".to_string()
                }
            }
            Action::Promotion {
                s_y,
                s_x,
                e_y,
                e_x,
                to_piece,
            } => {
                let piece_color = self.get_main_piece(state).unwrap().c;

                let s_row = to_row(s_y);
                let s_column = to_column(s_x);
                let e_row = to_row(e_y);
                let e_column = to_column(e_x);
                let promo_char = Piece {
                    c: piece_color,
                    t: to_piece,
                }
                .piece_to_char();

                let x_if_capture = match state.board[e_y][e_x] {
                    None => "",
                    Some(_) => "x",
                };

                format!(
                    "{} {}{}{}{}{}{}",
                    piece_char, s_column, x_if_capture, s_row, e_column, e_row, promo_char
                )
            }
            Action::Enpassant { s_y, s_x, e_y, e_x } => {
                let s_row = to_row(s_y);
                let s_column = to_column(s_x);
                let e_row = to_row(e_y);
                let e_column = to_column(e_x);

                format!(
                    "{} {}{}x{}{} e.p.",
                    piece_char, s_column, s_row, e_column, e_row
                )
            }
            Action::Tie => "½–½".to_string(),
        }
    }
}
