use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

use fxhash::FxHashMap;

use core::cmp::Ordering;

use crate::action::Action;
use crate::actions::actions;
use crate::result::{inplace_result, inplace_revert};
use crate::state::{State, PC};
use crate::value::{heuristic, heuristic_action, value, Status};

const INF: f64 = 1e+9;
const TIMEOUT: f64 = 2e+9;

fn minimax(
    state: &mut State,
    depth_left: u32,
    mut alpha: f64,
    mut beta: f64,
    heuristic_cache: f64,
    done_channel: &mut Receiver<()>,
    move_cache: &mut FxHashMap<u64, (u32, f64, Option<Action>)>,
) -> (f64, Option<Action>) {
    let (mut best_utility, mut best_action) = (-2. * INF, None);

    if depth_left >= 5 {
        if let Ok(()) = done_channel.try_recv() {
            return (TIMEOUT, None);
        }
    }

    let mut cache_best_action = None;

    if depth_left >= 1 {
        if let Some((cache_depth_left, cache_value, cache_action)) = move_cache.get(&state.hash()) {
            if cache_depth_left >= &depth_left {
                return (*cache_value, None);
            }
            cache_best_action = *cache_action;
        }
    }

    match value(state) {
        Status::Running => (),
        Status::BlackWin => {
            if state.turn == PC::Black {
                return (INF, None);
            } else {
                return (-INF, None);
            }
        }
        Status::WhiteWin => {
            if state.turn == PC::White {
                return (INF, None);
            } else {
                return (-INF, None);
            }
        }
        Status::Tie => {
            return (0., None);
        }
    }

    if depth_left == 0 {
        return (heuristic_cache, None);
    }

    let mut current_actions = actions(state);
    current_actions.sort_by(|a, b| {
        // compare so that array is in descending order
        let heuristic_comparison = heuristic_action(state, b)
            .partial_cmp(&heuristic_action(state, a))
            .unwrap();

        match &cache_best_action {
            Some(cache_action) => {
                if a == cache_action && b == cache_action {
                    Ordering::Equal
                } else if a == cache_action {
                    Ordering::Less
                } else if b == cache_action {
                    Ordering::Greater
                } else {
                    heuristic_comparison
                }
            }
            _ => heuristic_comparison,
        }
    });

    for a in current_actions {
        let action_heuristic = heuristic_action(state, &a);

        let moved_pieces = inplace_result(state, &a);

        let (mut response_utility, _) = minimax(
            state,
            depth_left - 1,
            alpha,
            beta,
            -(heuristic_cache + action_heuristic),
            done_channel,
            move_cache,
        );
        if response_utility == TIMEOUT {
            return (TIMEOUT, best_action);
        }
        response_utility *= -1.;
        if best_utility < response_utility {
            best_utility = response_utility;
            best_action = Some(a);
        }

        inplace_revert(state, moved_pieces);

        if state.turn == PC::White {
            alpha = f64::max(alpha, response_utility);
        } else {
            beta = f64::min(beta, -response_utility);
        }

        if alpha >= beta {
            break;
        }
    }

    if depth_left >= 1 {
        move_cache.insert(state.hash(), (depth_left, best_utility, best_action));
    }

    (best_utility, best_action)
}

pub fn ai_move(
    mut state: State,
    tx: Sender<(u32, Action)>,
    max_depth: u32,
    mut done_channel: Receiver<()>,
    move_cache: &mut FxHashMap<u64, (u32, f64, Option<Action>)>,
) {
    let start = Instant::now();
    for depth in 1..=max_depth {
        let curr_h = heuristic(&state);
        let (best_utility, best_action) = minimax(
            &mut state,
            depth,
            -INF,
            INF,
            curr_h,
            &mut done_channel,
            move_cache,
        );

        if done_channel.try_recv().is_ok() {
            return;
        }

        if best_utility != TIMEOUT {
            let best_action = best_action.expect("No move available for AI");
            println!(
                "AI move: {} at depth {}, with utility {:.2}",
                best_action.to_string(&state),
                depth,
                best_utility
            );
            let _ = tx.send((depth, best_action));
        }
    }
    let duration = start.elapsed();
    println!("Time elapsed for move is: {:?}", duration);
}
