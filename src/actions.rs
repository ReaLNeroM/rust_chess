use crate::action::Action;
use crate::result::result;
use crate::state::{Piece, State, PC, PT};

use std::cmp;
use std::convert::TryFrom;
use std::convert::TryInto;

fn valid_pawn_jump(state: &State, s_y: usize, s_x: usize, e_y: usize, e_x: usize) -> bool {
    let main_piece = state.board[s_y][s_x].unwrap();

    if main_piece.c == PC::Black {
        (e_y == 3 && s_y == 1 && state.board[2][s_x].is_none() && e_x == s_x)
            || (e_y == s_y + 1 && e_x == s_x)
    } else {
        (e_y == 4 && s_y == 6 && state.board[5][s_x].is_none() && e_x == s_x)
            || (e_y + 1 == s_y && e_x == s_x)
    }
}

fn valid_pawn_capture(state: &State, s_y: usize, s_x: usize, e_y: usize, e_x: usize) -> bool {
    let main_piece = state.board[s_y][s_x].unwrap();

    if main_piece.c == PC::Black {
        e_y == s_y + 1 && (e_x + 1 == s_x || e_x == s_x + 1)
    } else {
        e_y + 1 == s_y && (e_x + 1 == s_x || e_x == s_x + 1)
    }
}

fn valid_knight_jump(s_y: usize, s_x: usize, e_y: usize, e_x: usize) -> bool {
    let (y_delta, x_delta) = (e_y as isize - s_y as isize, e_x as isize - s_x as isize);
    let (y_delta_abs, x_delta_abs) = (y_delta.abs(), x_delta.abs());
    let (sorted_delta_1, sorted_delta_2) = {
        if y_delta_abs <= x_delta_abs {
            (y_delta_abs, x_delta_abs)
        } else {
            (x_delta_abs, y_delta_abs)
        }
    };

    sorted_delta_1 == 1 && sorted_delta_2 == 2
}

fn valid_bishop_jump(state: &State, s_y: usize, s_x: usize, e_y: usize, e_x: usize) -> bool {
    let d_y = if s_y < e_y { 1 } else { -1 };
    let d_x = if s_x < e_x { 1 } else { -1 };
    let steps = (e_y as isize - s_y as isize).abs();

    if e_y as isize != s_y as isize + d_y * steps
        || e_x as isize != s_x as isize + d_x as isize * steps
    {
        return false;
    }

    for i in 1..steps {
        let (c_y, c_x) = (
            (s_y as isize + d_y as isize * i) as usize,
            (s_x as isize + d_x as isize * i) as usize,
        );
        if state.board[c_y][c_x].is_some() {
            return false;
        }
    }

    true
}

fn valid_rook_jump(state: &State, s_y: usize, s_x: usize, e_y: usize, e_x: usize) -> bool {
    if e_y == s_y {
        let d_x = if s_x < e_x { 1 } else { -1 };
        let steps = (e_x as isize - s_x as isize).abs();
        for i in 1..steps {
            let c_x = (s_x as isize + d_x as isize * i) as usize;
            if state.board[e_y][c_x].is_some() {
                return false;
            }
        }

        true
    } else if e_x == s_x {
        let d_y = if s_y < e_y { 1 } else { -1 };
        let steps = (e_y as isize - s_y as isize).abs();
        for i in 1..steps {
            let c_y = (s_y as isize + d_y as isize * i) as usize;
            if state.board[c_y][e_x].is_some() {
                return false;
            }
        }

        true
    } else {
        false
    }
}

fn valid_queen_jump(state: &State, s_y: usize, s_x: usize, e_y: usize, e_x: usize) -> bool {
    valid_rook_jump(state, s_y, s_x, e_y, e_x) || valid_bishop_jump(state, s_y, s_x, e_y, e_x)
}

fn valid_king_jump(s_y: usize, s_x: usize, e_y: usize, e_x: usize) -> bool {
    cmp::max(
        (e_y as isize - s_y as isize).abs(),
        (e_x as isize - s_x as isize).abs(),
    ) == 1
}

fn validate_action_ignore_check(state: &State, action: &Action) -> bool {
    let main_piece = match action.get_main_piece(state) {
        Some(piece) => piece,
        None => {
            return false;
        }
    };

    if main_piece.c != state.turn {
        return false;
    }
    match *action {
        Action::Jump { s_y, s_x, e_y, e_x } => {
            if state.board[s_y][s_x].is_none() {
                return false;
            }
            if state.board[e_y][e_x].is_some() {
                return false;
            }

            match main_piece.t {
                PT::Pawn { .. } => valid_pawn_jump(state, s_y, s_x, e_y, e_x),
                PT::Knight => valid_knight_jump(s_y, s_x, e_y, e_x),
                PT::Bishop => valid_bishop_jump(state, s_y, s_x, e_y, e_x),
                PT::Rook { .. } => valid_rook_jump(state, s_y, s_x, e_y, e_x),
                PT::Queen => valid_queen_jump(state, s_y, s_x, e_y, e_x),
                PT::King { .. } => valid_king_jump(s_y, s_x, e_y, e_x),
            }
        }
        Action::Capture { s_y, s_x, e_y, e_x } => {
            // check if jump is valid.
            let capture_piece = match state.board[e_y][e_x] {
                Some(p) => p,
                None => {
                    return false;
                }
            };
            if main_piece.c == capture_piece.c {
                return false;
            }

            match main_piece.t {
                PT::Pawn { .. } => valid_pawn_capture(state, s_y, s_x, e_y, e_x),
                PT::Knight => valid_knight_jump(s_y, s_x, e_y, e_x),
                PT::Bishop => valid_bishop_jump(state, s_y, s_x, e_y, e_x),
                PT::Rook { .. } => valid_rook_jump(state, s_y, s_x, e_y, e_x),
                PT::Queen => valid_queen_jump(state, s_y, s_x, e_y, e_x),
                PT::King { .. } => valid_king_jump(s_y, s_x, e_y, e_x),
            }
        }
        Action::Castling {
            s_y,
            s_x,
            queenside,
        } => {
            match main_piece.t {
                PT::King { has_moved: false } => (),
                _ => return false,
            }
            if s_y != 0 && s_y != 7 {
                return false;
            }
            if queenside {
                // row s_y must look like: R...K???
                match state.board[s_y][0] {
                    Some(Piece {
                        c,
                        t: PT::Rook { has_moved: false },
                    }) if c == state.turn => (),
                    _ => {
                        return false;
                    }
                }
                for i in 1..s_x {
                    if state.board[s_y][i].is_some() {
                        return false;
                    }
                }
                // note: validate_action_ignore_check calls is_attacked, then is_attacked calls
                //       validate_.... This is fine, since validate_... only calls is_attacked when
                //       talking about a castling move.
                for i in 0..=s_x {
                    if is_attacked(state, (s_y, i), state.turn) {
                        return false;
                    }
                }

                true
            } else {
                // row s_y must look like: ????K..R
                match state.board[s_y][7] {
                    Some(Piece {
                        c,
                        t: PT::Rook { has_moved: false },
                    }) if c == state.turn => (),
                    _ => {
                        return false;
                    }
                }
                for i in (s_x + 1)..7 {
                    if state.board[s_y][i].is_some() {
                        return false;
                    }
                }
                // same note applies here as well.
                for i in s_x..=7 {
                    if is_attacked(state, (s_y, i), state.turn) {
                        return false;
                    }
                }

                true
            }
        }
        Action::Promotion {
            s_y,
            s_x,
            e_y,
            e_x,
            to_piece,
        } => {
            match main_piece.t {
                PT::Pawn { .. } => (),
                _ => {
                    return false;
                }
            }
            match to_piece {
                PT::Bishop => (),
                PT::Rook { has_moved: true } => (),
                PT::Knight => (),
                PT::Queen => (),
                _ => {
                    return false;
                }
            }
            if (main_piece.c == PC::Black && e_y != 7) || (main_piece.c == PC::White && e_y != 0) {
                return false;
            }
            if state.board[e_y][e_x].is_some() {
                valid_pawn_capture(state, s_y, s_x, e_y, e_x)
            } else {
                valid_pawn_jump(state, s_y, s_x, e_y, e_x)
            }
        }
        Action::Enpassant { s_y, s_x, e_y, e_x } => {
            let capture_piece = match state.board[s_y][e_x] {
                Some(p) => p,
                None => {
                    return false;
                }
            };
            if main_piece.c == capture_piece.c {
                return false;
            }

            match main_piece.t {
                PT::Pawn { .. } => (),
                _ => {
                    return false;
                }
            }
            match capture_piece.t {
                PT::Pawn { last_jump } => {
                    if state.moves != 0 && last_jump != Some(state.moves - 1) {
                        return false;
                    }
                }
                _ => {
                    return false;
                }
            }

            match main_piece.c {
                PC::Black => s_y == 4 && e_y == 5 && (e_x + 1 == s_x || e_x == s_x + 1),
                PC::White => s_y == 3 && e_y == 2 && (e_x + 1 == s_x || e_x == s_x + 1),
            }
        }
        Action::Tie => {
            let hash = state.hash();

            match state.hash_to_occurences.get(&hash) {
                None => false,
                Some(x) => *x >= 3,
            }
        }
    }
}

// ONLY used for castling/king.
fn is_attacked(state: &State, position: (usize, usize), me: PC) -> bool {
    let (e_y, e_x) = position;
    for s_y in 0..8 {
        for s_x in 0..8 {
            if let Some(p) = state.board[s_y][s_x] {
                let is_opponent_piece = p.c == me.opponent();
                let can_piece_capture = match p.t {
                    PT::Pawn { .. } => valid_pawn_capture(state, s_y, s_x, e_y, e_x),
                    PT::Knight => valid_knight_jump(s_y, s_x, e_y, e_x),
                    PT::Bishop => valid_bishop_jump(state, s_y, s_x, e_y, e_x),
                    PT::Rook { .. } => valid_rook_jump(state, s_y, s_x, e_y, e_x),
                    PT::Queen => valid_queen_jump(state, s_y, s_x, e_y, e_x),
                    PT::King { .. } => valid_king_jump(s_y, s_x, e_y, e_x),
                };

                if is_opponent_piece && can_piece_capture {
                    return true;
                }
            }
        }
    }

    false
}

pub fn is_king_attacked(state: &State, me: PC) -> bool {
    let (ky, kx) = {
        let mut k = None;

        'columns: for i in 0..8 {
            for j in 0..8 {
                if let Some(Piece {
                    c,
                    t: PT::King { .. },
                }) = state.board[i][j]
                {
                    if c == me {
                        k = Some((i, j));
                        break 'columns;
                    }
                }
            }
        }

        match k {
            Some(k) => k,
            None => panic!("Could not find king in board."),
        }
    };

    is_attacked(state, (ky, kx), me)
}

pub fn validate_action(state: &State, action: &Action) -> bool {
    if !validate_action_ignore_check(state, action) {
        return false;
    }

    let after_state = result(state, action);
    !is_king_attacked(&after_state, state.turn)
}

fn within_bounds(y: isize, x: isize) -> Option<(usize, usize)> {
    if y >= 8 || x >= 8 {
        None
    } else {
        let y = usize::try_from(y).ok();
        let x = usize::try_from(x).ok();

        y.zip(x)
    }
}

fn interesting_locations(s_y: usize, s_x: usize, piece: &Piece) -> Vec<(usize, usize)> {
    let deltas = match piece.t {
        PT::Pawn { .. } => {
            vec![
                (-1, -1),
                (-1, 1),
                (1, -1),
                (1, 1),
                (1, 0),
                (-1, 0),
                (2, 0),
                (-2, 0),
            ]
        }
        PT::Knight => {
            vec![
                (-2, -1),
                (-2, 1),
                (-1, -2),
                (-1, 2),
                (1, -2),
                (1, 2),
                (2, -1),
                (2, 1),
            ]
        }
        PT::Bishop => {
            let mut deltas = vec![];
            for i in 1..8 {
                deltas.push((-i, -i));
                deltas.push((-i, i));
                deltas.push((i, -i));
                deltas.push((i, i));
            }
            deltas
        }
        PT::Rook { .. } => {
            let mut deltas = vec![];
            for i in 1..8 {
                deltas.push((-i, 0));
                deltas.push((0, -i));
                deltas.push((0, i));
                deltas.push((i, 0));
            }
            deltas
        }
        PT::Queen => {
            let mut deltas = vec![];
            for i in 1..8 {
                deltas.push((-i, -i));
                deltas.push((-i, i));
                deltas.push((i, -i));
                deltas.push((i, i));
            }
            for i in 1..8 {
                deltas.push((-i, 0));
                deltas.push((0, -i));
                deltas.push((0, i));
                deltas.push((i, 0));
            }

            deltas
        }
        PT::King { .. } => {
            vec![
                (-1, -1),
                (-1, 0),
                (-1, 1),
                (0, -1),
                (0, 1),
                (1, -1),
                (1, 0),
                (1, 1),
                (0, 2),
                (0, -2),
            ]
        }
    };

    deltas
        .iter()
        .map(|(dy, dx)| (s_y as isize + dy, s_x as isize + dx))
        .filter_map(|(sy, sx)| within_bounds(sy, sx))
        .collect()
}

pub fn actions_for_location(state: &State, s_y: usize, s_x: usize) -> Vec<Action> {
    let mut found = vec![];
    for e_y in 0..8 {
        for e_x in 0..8 {
            if let Some(action) =
                Action::from_context_and_coords(state, (s_y, s_x), (e_y, e_x), None)
            {
                if validate_action(state, &action) {
                    found.push(action);
                }
            }
        }
    }

    let possible_promotion_types = vec![
        PT::Queen,
        PT::Rook { has_moved: true },
        PT::Bishop,
        PT::Knight,
    ];

    for pt in possible_promotion_types {
        for e_y in [0, 7].iter() {
            for d_x in -1..=1 {
                let e_x = s_x as isize + d_x;
                match e_x.try_into().ok() {
                    None => (),
                    Some(e_x) => {
                        if e_x < 8 {
                            let current_action = Action::Promotion {
                                s_y,
                                s_x,
                                e_y: *e_y,
                                e_x,
                                to_piece: pt,
                            };
                            if validate_action(state, &current_action) {
                                found.push(current_action);
                            }
                        }
                    }
                }
            }
        }
    }

    found
}

pub fn actions(state: &State) -> Vec<Action> {
    let mut found = vec![];
    for s_y in 0..8 {
        for s_x in 0..8 {
            let piece = match state.board[s_y][s_x] {
                Some(p) => {
                    if p.c != state.turn {
                        continue;
                    }
                    p
                }
                None => {
                    continue;
                }
            };

            for (e_y, e_x) in interesting_locations(s_y, s_x, &piece) {
                if let Some(action) =
                    Action::from_context_and_coords(state, (s_y, s_x), (e_y, e_x), None)
                {
                    if validate_action(state, &action) {
                        found.push(action);
                    }
                }
            }
        }
    }

    let possible_promotion_types = vec![
        PT::Queen,
        PT::Rook { has_moved: true },
        PT::Bishop,
        PT::Knight,
    ];

    for s_x in 0..8 {
        for pt in &possible_promotion_types {
            for (e_y, s_y) in [(0, 1), (6, 7)].iter() {
                for d_x in -1..=1 {
                    let e_x = s_x as isize + d_x;
                    match e_x.try_into().ok() {
                        None => (),
                        Some(e_x) => {
                            if e_x < 8 {
                                let current_action = Action::Promotion {
                                    s_y: *s_y,
                                    s_x,
                                    e_y: *e_y,
                                    e_x,
                                    to_piece: *pt,
                                };
                                if validate_action(state, &current_action) {
                                    found.push(current_action);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    match state.hash_to_occurences.get(&state.hash()) {
        None => (),
        Some(&x) => {
            if x >= 3 {
                found.push(Action::Tie);
            }
        }
    }

    found
}

pub fn any_actions(state: &State) -> bool {
    for s_y in 0..8 {
        for s_x in 0..8 {
            let piece = match state.board[s_y][s_x] {
                Some(p) => {
                    if p.c != state.turn {
                        continue;
                    }
                    p
                }
                None => {
                    continue;
                }
            };

            for (e_y, e_x) in interesting_locations(s_y, s_x, &piece) {
                if let Some(action) =
                    Action::from_context_and_coords(state, (s_y, s_x), (e_y, e_x), None)
                {
                    if validate_action(state, &action) {
                        return true;
                    }
                }
            }
        }
    }

    let possible_promotion_types = vec![
        PT::Queen,
        PT::Rook { has_moved: true },
        PT::Bishop,
        PT::Knight,
    ];

    for s_x in 0..8 {
        for pt in &possible_promotion_types {
            for (e_y, s_y) in [(0, 1), (6, 7)].iter() {
                for d_x in -1..=1 {
                    let e_x = s_x as isize + d_x;
                    match e_x.try_into().ok() {
                        None => (),
                        Some(e_x) => {
                            if e_x < 8 {
                                let current_action = Action::Promotion {
                                    s_y: *s_y,
                                    s_x,
                                    e_y: *e_y,
                                    e_x,
                                    to_piece: *pt,
                                };
                                if validate_action(state, &current_action) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    match state.hash_to_occurences.get(&state.hash()) {
        None => (),
        Some(&x) => {
            if x >= 3 {
                return true;
            }
        }
    }

    false
}
