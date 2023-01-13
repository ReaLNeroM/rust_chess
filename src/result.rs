use crate::action::Action;
use crate::state::{Piece, State, PT};

pub fn inplace_result(
    mut state: &mut State,
    action: &Action,
) -> Vec<(usize, usize, Option<Piece>)> {
    let mut moved_pieces = Vec::new();

    fn piece_with_updated_piece_state(state: &mut State, e_y: usize, e_x: usize, s_y: usize) {
        match state.board[e_y][e_x].as_mut() {
            None => (),
            Some(p) => match p.t {
                PT::Pawn { ref mut last_jump } => {
                    let updated_last_jump = if s_y == e_y + 2 || e_y == s_y + 2 {
                        Some(state.moves)
                    } else {
                        None
                    };

                    *last_jump = updated_last_jump;
                }
                PT::Rook { ref mut has_moved } => {
                    *has_moved = true;
                }
                PT::King { ref mut has_moved } => {
                    *has_moved = true;
                }
                _ => (),
            },
        }
    }

    match *action {
        Action::Jump { s_y, s_x, e_y, e_x } => {
            moved_pieces.push((s_y, s_x, state.board[s_y][s_x]));
            moved_pieces.push((e_y, e_x, None));

            state.board[e_y][e_x] = state.board[s_y][s_x];
            piece_with_updated_piece_state(state, e_y, e_x, s_y);
            state.board[s_y][s_x] = None;
        }
        Action::Capture { s_y, s_x, e_y, e_x } => {
            moved_pieces.push((s_y, s_x, state.board[s_y][s_x]));
            moved_pieces.push((e_y, e_x, state.board[e_y][e_x]));

            state.board[e_y][e_x] = state.board[s_y][s_x];
            piece_with_updated_piece_state(state, e_y, e_x, s_y);
            state.board[s_y][s_x] = None;
        }
        Action::Castling {
            s_y,
            s_x,
            queenside,
        } => {
            if queenside {
                moved_pieces.push((s_y, 2, state.board[s_y][2]));
                moved_pieces.push((s_y, s_x, state.board[s_y][s_x]));
                moved_pieces.push((s_y, 3, state.board[s_y][3]));
                moved_pieces.push((s_y, 0, state.board[s_y][0]));

                state.board[s_y][2] = state.board[s_y][s_x];
                piece_with_updated_piece_state(state, s_y, 2, s_y);
                state.board[s_y][s_x] = None;

                state.board[s_y][3] = state.board[s_y][0];
                piece_with_updated_piece_state(state, s_y, 3, s_y);
                state.board[s_y][0] = None;
            } else {
                moved_pieces.push((s_y, 6, state.board[s_y][6]));
                moved_pieces.push((s_y, s_x, state.board[s_y][s_x]));
                moved_pieces.push((s_y, 5, state.board[s_y][5]));
                moved_pieces.push((s_y, 7, state.board[s_y][7]));

                state.board[s_y][6] = state.board[s_y][s_x];
                piece_with_updated_piece_state(state, s_y, 6, s_y);
                state.board[s_y][s_x] = None;

                state.board[s_y][5] = state.board[s_y][7];
                piece_with_updated_piece_state(state, s_y, 5, s_y);
                state.board[s_y][7] = None;
            }
        }
        Action::Promotion {
            s_y,
            s_x,
            e_y,
            e_x,
            to_piece,
        } => {
            moved_pieces.push((s_y, s_x, state.board[s_y][s_x]));
            moved_pieces.push((e_y, e_x, state.board[e_y][e_x]));

            state.board[e_y][e_x] = Some(Piece {
                c: state.turn,
                t: to_piece,
            });
            state.board[s_y][s_x] = None;
        }
        Action::Enpassant { s_y, s_x, e_y, e_x } => {
            moved_pieces.push((s_y, s_x, state.board[s_y][s_x]));
            moved_pieces.push((e_y, e_x, state.board[e_y][e_x]));
            moved_pieces.push((s_y, e_x, state.board[s_y][e_x]));

            state.board[e_y][e_x] = state.board[s_y][s_x];
            piece_with_updated_piece_state(state, e_y, e_x, s_y);
            state.board[s_y][s_x] = None;
            state.board[s_y][e_x] = None;
        }
        Action::Tie => state.drawn = true,
    };

    state.turn = state.turn.opponent();
    state.moves += 1;

    let v = state.hash_to_occurences.entry(state.hash()).or_insert(0);

    *v += 1;
    if *v >= 5 {
        state.drawn = true;
    }

    moved_pieces
}

pub fn inplace_revert(mut state: &mut State, moved_pieces: Vec<(usize, usize, Option<Piece>)>) {
    match state.hash_to_occurences.remove(&state.hash()) {
        None | Some(1) => (),
        Some(x) => {
            state.hash_to_occurences.insert(state.hash(), x - 1);
        }
    };

    for (y, x, p) in moved_pieces {
        state.board[y][x] = p;
    }

    state.drawn = false;
    state.turn = state.turn.opponent();
    state.moves -= 1;
}

// Note: this function doesn't check if the action is valid.
// Applying the action when it is invalid leads to undefined behavior.
pub fn result(old_state: &State, action: &Action) -> State {
    let mut new_state = old_state.clone();

    new_state.turn = new_state.turn.opponent();
    new_state.moves += 1;

    let piece_with_updated_piece_state =
        |state: &mut State, e_y: usize, e_x: usize, s_y: usize| match &state.board[e_y][e_x] {
            Some(Piece {
                t: PT::Pawn { .. },
                c,
            }) => {
                let last_jump = if e_y == s_y + 2 || s_y == e_y + 2 {
                    Some(old_state.moves)
                } else {
                    None
                };

                Some(Piece {
                    t: PT::Pawn { last_jump },
                    c: *c,
                })
            }
            Some(Piece {
                t: PT::Rook { .. },
                c,
            }) => Some(Piece {
                t: PT::Rook { has_moved: true },
                c: *c,
            }),
            Some(Piece {
                t: PT::King { .. },
                c,
            }) => Some(Piece {
                t: PT::King { has_moved: true },
                c: *c,
            }),
            Some(Piece { t, c }) => Some(Piece { t: *t, c: *c }),
            None => None,
        };

    match *action {
        Action::Jump { s_y, s_x, e_y, e_x } => {
            new_state.board[e_y][e_x] = new_state.board[s_y][s_x];
            new_state.board[e_y][e_x] =
                piece_with_updated_piece_state(&mut new_state, e_y, e_x, s_y);
            new_state.board[s_y][s_x] = None;
        }
        Action::Capture { s_y, s_x, e_y, e_x } => {
            new_state.board[e_y][e_x] = new_state.board[s_y][s_x];
            new_state.board[e_y][e_x] =
                piece_with_updated_piece_state(&mut new_state, e_y, e_x, s_y);
            new_state.board[s_y][s_x] = None;
        }
        Action::Castling {
            s_y,
            s_x,
            queenside,
        } => {
            if queenside {
                new_state.board[s_y][2] = new_state.board[s_y][s_x];
                new_state.board[s_y][2] =
                    piece_with_updated_piece_state(&mut new_state, s_y, 2, s_y);
                new_state.board[s_y][s_x] = None;

                new_state.board[s_y][3] = new_state.board[s_y][0];
                new_state.board[s_y][3] =
                    piece_with_updated_piece_state(&mut new_state, s_y, 3, s_y);
                new_state.board[s_y][0] = None;
            } else {
                new_state.board[s_y][6] = new_state.board[s_y][s_x];
                new_state.board[s_y][6] =
                    piece_with_updated_piece_state(&mut new_state, s_y, 6, s_y);
                new_state.board[s_y][s_x] = None;

                new_state.board[s_y][5] = new_state.board[s_y][7];
                new_state.board[s_y][5] =
                    piece_with_updated_piece_state(&mut new_state, s_y, 5, s_y);
                new_state.board[s_y][7] = None;
            }
        }
        Action::Promotion {
            s_y,
            s_x,
            e_y,
            e_x,
            to_piece,
        } => {
            new_state.board[e_y][e_x] = Some(Piece {
                c: old_state.turn,
                t: to_piece,
            });
            new_state.board[s_y][s_x] = None;
        }
        Action::Enpassant { s_y, s_x, e_y, e_x } => {
            new_state.board[e_y][e_x] = new_state.board[s_y][s_x];
            new_state.board[e_y][e_x] =
                piece_with_updated_piece_state(&mut new_state, e_y, e_x, s_y);
            new_state.board[s_y][s_x] = None;
            new_state.board[s_y][e_x] = None;
        }
        Action::Tie => {
            new_state.drawn = true;
        }
    };

    let v = new_state
        .hash_to_occurences
        .entry(new_state.hash())
        .or_insert(0);

    *v += 1;
    if *v >= 5 {
        new_state.drawn = true;
    }

    new_state
}
