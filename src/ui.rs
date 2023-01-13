use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use fxhash::FxHashMap;
use sfml::audio::{Sound, SoundBuffer};
use sfml::graphics::Transformable;
use sfml::graphics::{CircleShape, Color, RenderTarget, RenderWindow, Shape, Sprite, Texture};
use sfml::window::mouse::Button;
use sfml::window::{ContextSettings, Event, Key, Style, VideoMode};
use sfml::SfBox;

use crate::action::Action;
use crate::actions::{actions_for_location, validate_action};
use crate::ai::ai_move;
use crate::result::result;
use crate::state::{Piece, State, PC, PT};
use crate::value::{value, Status};

enum UiState {
    PlayerThinking,
    PlayerHighlighted {
        y: usize,
        x: usize,
        targets: Vec<Action>,
    },
    AIThinking(Receiver<(u32, Action)>, Sender<()>),
    Done,
}

pub enum Thinker {
    Player,
    AI,
}

impl Thinker {
    fn to_ui_state(&self, state: &State, ai_lookahead_depth: u32) -> UiState {
        match self {
            Thinker::Player => UiState::PlayerThinking,
            Thinker::AI => {
                let (tx, done_rx) = launch_ai_thread(state.clone(), ai_lookahead_depth);
                UiState::AIThinking(tx, done_rx)
            }
        }
    }
}

struct ChessTextures {
    checkerboard_texture: SfBox<Texture>,
    piece_textures: [SfBox<Texture>; 12],
}

impl ChessTextures {
    fn new() -> Self {
        let checkerboard_texture = Texture::from_file("img/board.png").unwrap();
        let mut piece_textures = [
            Texture::from_file("img/black_pawn.png").unwrap(),
            Texture::from_file("img/black_knight.png").unwrap(),
            Texture::from_file("img/black_bishop.png").unwrap(),
            Texture::from_file("img/black_rook.png").unwrap(),
            Texture::from_file("img/black_queen.png").unwrap(),
            Texture::from_file("img/black_king.png").unwrap(),
            Texture::from_file("img/white_pawn.png").unwrap(),
            Texture::from_file("img/white_knight.png").unwrap(),
            Texture::from_file("img/white_bishop.png").unwrap(),
            Texture::from_file("img/white_rook.png").unwrap(),
            Texture::from_file("img/white_queen.png").unwrap(),
            Texture::from_file("img/white_king.png").unwrap(),
        ];

        for i in &mut piece_textures {
            i.deref_mut().set_smooth(true);
        }
        Self {
            checkerboard_texture,
            piece_textures,
        }
    }
    fn get_texture(&self, p: &Piece) -> &Texture {
        match p {
            Piece {
                c: PC::Black,
                t: PT::Pawn { .. },
            } => &self.piece_textures[0],
            Piece {
                c: PC::Black,
                t: PT::Knight,
            } => &self.piece_textures[1],
            Piece {
                c: PC::Black,
                t: PT::Bishop,
            } => &self.piece_textures[2],
            Piece {
                c: PC::Black,
                t: PT::Rook { .. },
            } => &self.piece_textures[3],
            Piece {
                c: PC::Black,
                t: PT::Queen,
            } => &self.piece_textures[4],
            Piece {
                c: PC::Black,
                t: PT::King { .. },
            } => &self.piece_textures[5],
            Piece {
                c: PC::White,
                t: PT::Pawn { .. },
            } => &self.piece_textures[6],
            Piece {
                c: PC::White,
                t: PT::Knight,
            } => &self.piece_textures[7],
            Piece {
                c: PC::White,
                t: PT::Bishop,
            } => &self.piece_textures[8],
            Piece {
                c: PC::White,
                t: PT::Rook { .. },
            } => &self.piece_textures[9],
            Piece {
                c: PC::White,
                t: PT::Queen,
            } => &self.piece_textures[10],
            Piece {
                c: PC::White,
                t: PT::King { .. },
            } => &self.piece_textures[11],
        }
    }
    fn get_checkerboard_texture(&self) -> &Texture {
        &self.checkerboard_texture
    }
}

fn launch_ai_thread(state: State, max_depth: u32) -> (Receiver<(u32, Action)>, Sender<()>) {
    let (tx, rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    let mut move_cache = FxHashMap::default();

    thread::spawn(move || {
        ai_move(state, tx, max_depth, done_rx, &mut move_cache);
    });
    (rx, done_tx)
}

fn get_board_coordinates(coords: (u32, u32), window_size: u32) -> Option<(usize, usize)> {
    let (x, y) = coords;
    if x >= window_size || y >= window_size {
        return None;
    }

    let (bx, by) = (x * 8 / window_size, y * 8 / window_size);

    if bx >= 8 || by >= 8 {
        None
    } else {
        Some((bx as usize, by as usize))
    }
}

pub fn ui_routine(color_assignments: HashMap<PC, Thinker>) {
    let mut history = vec![State::new()];

    const AI_LOOKAHEAD_DEPTH: u32 = 7;

    const WINDOWSIZE: u32 = 1200;
    let cell_size = WINDOWSIZE as f32 / 8.;

    let context_settings = ContextSettings {
        antialiasing_level: 2,
        ..Default::default()
    };
    let mut window = RenderWindow::new(
        VideoMode::new(WINDOWSIZE, WINDOWSIZE, 32),
        "Vlad's Chess",
        Style::CLOSE,
        &context_settings,
    );
    window.set_framerate_limit(120);

    let buffer = SoundBuffer::from_file("media/Move.ogg").unwrap();
    let mut sound = Sound::with_buffer(&buffer);

    let chess_textures = ChessTextures::new();
    let generate_sprite_board = |state: &State| {
        let mut sprite_board: Vec<Vec<Option<Sprite>>> = vec![vec![None; 8]; 8];
        for (i, sr) in sprite_board.iter_mut().enumerate() {
            for (j, s) in sr.iter_mut().enumerate() {
                *s = if let Some(p) = state.board[i][j] {
                    let t = chess_textures.get_texture(&p);
                    let mut s = Sprite::with_texture(t);
                    s.set_position((j as f32 * cell_size, i as f32 * cell_size));
                    s.set_scale((cell_size / t.size().x as f32, cell_size / t.size().y as f32));
                    Some(s)
                } else {
                    None
                }
            }
        }
        sprite_board
    };
    let checkerboard_sprite = {
        let mut s = Sprite::with_texture(chess_textures.get_checkerboard_texture());
        s.set_scale((
            WINDOWSIZE as f32 / chess_textures.get_checkerboard_texture().size().x as f32,
            WINDOWSIZE as f32 / chess_textures.get_checkerboard_texture().size().y as f32,
        ));
        s
    };
    let mut sprite_board = generate_sprite_board(&history[history.len() - 1]);

    let mut last_start = None;
    let mut last_end = None;

    let mut ui_state =
        color_assignments[&history[0].turn].to_ui_state(&history[0], AI_LOOKAHEAD_DEPTH);

    let mut is_player_done_waiting = false;
    let mut latest_move = None;

    'gameLoop: while window.is_open() {
        let display_state = &history[history.len() - 1].clone();
        // Handle events
        let mut release_locations = Vec::new();

        loop {
            match window.poll_event() {
                Some(Event::Closed) => window.close(),
                Some(Event::MouseButtonReleased {
                    button: Button::Left,
                    x,
                    y,
                }) => {
                    release_locations.push((y as u32, x as u32));
                }
                Some(Event::MouseButtonReleased {
                    button: Button::Right,
                    ..
                }) => {
                    // If the player is current, then we revert one move and let the player
                    // play for the AI

                    if history.len() > 1 {
                        if let UiState::AIThinking(_, done_tx) = &ui_state {
                            let _ = done_tx.send(());
                            is_player_done_waiting = false;
                        }
                        ui_state = UiState::PlayerThinking;

                        history.pop();
                        let old_state = &history[history.len() - 1];
                        sprite_board = generate_sprite_board(old_state);

                        last_start = None;
                        last_end = None;

                        continue 'gameLoop;
                    }
                }
                Some(Event::KeyPressed {
                    code: Key::Space, ..
                }) => {
                    is_player_done_waiting = true;
                }
                Some(_) => (),
                None => break,
            }
        }

        match ui_state {
            UiState::PlayerThinking => {
                for coords in release_locations {
                    match get_board_coordinates(coords, WINDOWSIZE) {
                        None => (),
                        Some((by, bx)) => {
                            if display_state.board[by][bx].is_some() {
                                ui_state = UiState::PlayerHighlighted {
                                    y: by,
                                    x: bx,
                                    targets: actions_for_location(display_state, by, bx),
                                };
                            }
                        }
                    }
                }
            }
            UiState::PlayerHighlighted { y, x, targets: _ } => {
                for coords in release_locations {
                    match get_board_coordinates(coords, WINDOWSIZE) {
                        None => {
                            ui_state = UiState::PlayerThinking;
                            continue;
                        }
                        Some((by, bx)) => {
                            if y == by && x == bx {
                                // remove highlight
                                ui_state = UiState::PlayerThinking;
                                continue;
                            }

                            // perform move
                            let start_coords = (y, x);
                            let end_coords = (by, bx);

                            // special-case for promotion
                            let promotion_choice = if validate_action(
                                display_state,
                                &Action::Promotion {
                                    s_y: start_coords.0,
                                    s_x: start_coords.1,
                                    e_y: end_coords.0,
                                    e_x: end_coords.1,
                                    to_piece: PT::Queen,
                                },
                            ) {
                                use std::io::{stdin, stdout, Write};
                                let promotion_type;
                                loop {
                                    let mut s = String::new();
                                    print!("Enter promotion type (Q for Queen, R for Rook, B for Bishop, and K for Knight): ");
                                    let _ = stdout().flush();
                                    stdin()
                                        .read_line(&mut s)
                                        .expect("Did not enter a correct string");
                                    if let Some('\n') = s.chars().next_back() {
                                        s.pop();
                                    }
                                    if let Some('\r') = s.chars().next_back() {
                                        s.pop();
                                    }
                                    let pt = match &s[..] {
                                        "Q" => PT::Queen,
                                        "R" => PT::Rook { has_moved: true },
                                        "B" => PT::Bishop,
                                        "K" => PT::Knight,
                                        _ => {
                                            continue;
                                        }
                                    };
                                    promotion_type = pt;
                                    break;
                                }
                                Some(promotion_type)
                            } else {
                                None
                            };

                            if let Some(action) = Action::from_context_and_coords(
                                display_state,
                                start_coords,
                                end_coords,
                                promotion_choice,
                            ) {
                                if validate_action(display_state, &action) {
                                    println!("Player move: {}", action.to_string(display_state));
                                    let new_state = result(display_state, &action);
                                    log::info!("State: {:?}", new_state);
                                    last_start = Some(action.get_main_coords());
                                    last_end = Some(action.get_end_coords());

                                    sprite_board = generate_sprite_board(&new_state);
                                    ui_state = match value(&new_state) {
                                        Status::Running => color_assignments[&new_state.turn]
                                            .to_ui_state(&new_state, AI_LOOKAHEAD_DEPTH),
                                        Status::BlackWin => {
                                            println!("Black win!");
                                            UiState::Done
                                        }
                                        Status::WhiteWin => {
                                            println!("White win!");
                                            UiState::Done
                                        }
                                        Status::Tie => {
                                            println!("Tie!");
                                            UiState::Done
                                        }
                                    };

                                    sound.play();
                                    history.push(new_state);
                                    is_player_done_waiting = false;
                                } else {
                                    ui_state = UiState::PlayerThinking;
                                }
                            } else {
                                ui_state = UiState::PlayerThinking;
                            }
                        }
                    }
                }
            }
            UiState::AIThinking(ref tx, ref done_rx) => {
                while let Ok((moves_ahead, ai_action)) = tx.try_recv() {
                    latest_move = Some((moves_ahead, ai_action));
                }

                match latest_move {
                    None => (),
                    Some((moves_ahead, ai_action)) => {
                        if moves_ahead == AI_LOOKAHEAD_DEPTH || is_player_done_waiting {
                            is_player_done_waiting = false;
                            let _ = done_rx.send(());

                            let new_state = result(display_state, &ai_action);
                            last_start = Some(ai_action.get_main_coords());
                            last_end = Some(ai_action.get_end_coords());

                            ui_state = match value(&new_state) {
                                Status::Running => color_assignments[&new_state.turn]
                                    .to_ui_state(&new_state, AI_LOOKAHEAD_DEPTH),
                                Status::BlackWin => {
                                    println!("Black win!");
                                    UiState::Done
                                }
                                Status::WhiteWin => {
                                    println!("White win!");
                                    UiState::Done
                                }
                                Status::Tie => {
                                    println!("Tie!");
                                    UiState::Done
                                }
                            };

                            sprite_board = generate_sprite_board(&new_state);

                            sound.play();
                            history.push(new_state);
                        }
                    }
                }
            }
            UiState::Done => (),
        }

        // Clear the window
        window.clear(Color::rgb(0, 0, 0));

        window.draw(&checkerboard_sprite);

        if let Some(last_start_coords) = last_start {
            let (by, bx) = last_start_coords;
            let mut circle = CircleShape::new(cell_size / 2.0, 30);
            circle.set_fill_color(Color::rgba(0, 0, 128, 192));
            circle.set_position((bx as f32 * cell_size, by as f32 * cell_size));
            window.draw(&circle);
        }
        if let Some(last_end_coords) = last_end {
            let (by, bx) = last_end_coords;
            let mut circle = CircleShape::new(cell_size / 2.0, 30);
            circle.set_fill_color(Color::rgba(0, 0, 255, 192));
            circle.set_position((bx as f32 * cell_size, by as f32 * cell_size));
            window.draw(&circle);
        }
        if let UiState::PlayerHighlighted { y, x, ref targets } = ui_state {
            let mut circle = CircleShape::new(cell_size / 2.0, 30);
            circle.set_fill_color(Color::rgba(255, 0, 0, 192));
            circle.set_position((x as f32 * cell_size, y as f32 * cell_size));
            window.draw(&circle);

            for action in targets {
                let (by, bx) = action.get_end_coords();
                let mut circle = CircleShape::new(cell_size / 2.0, 30);
                circle.set_fill_color(Color::rgba(0, 255, 0, 192));
                circle.set_position((bx as f32 * cell_size, by as f32 * cell_size));
                window.draw(&circle);
            }
        }

        for i in &sprite_board {
            for j in i {
                match j {
                    Some(s) => window.draw(s),
                    None => (),
                }
            }
        }

        window.display();
    }
}
