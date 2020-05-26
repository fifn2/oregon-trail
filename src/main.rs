#![allow(dead_code)]

use chrono::prelude::*;
use chrono::Duration;
use redux_rs::{Store, Subscription};
use std::boxed::Box;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct State {
    date: Date<Utc>,
    miles: u64,
    food: u64,
    health: u64,
    hunt_days: i64,
}

enum Action<'a> {
    Help(Box<dyn Fn(State) -> State + 'a>),
    Hunt,
    Quit(Box<dyn Fn(State) -> State + 'a>),
    Rest(Duration),
    Status(Box<dyn Fn(State) -> State + 'a>),
    Travel(Duration, u64),
}

enum SimpleAction {
    Hunt,
    Travel(Duration, u64),
    Rest(Duration),
}

impl<'a> From<SimpleAction> for Action<'a> {
    fn from(action: SimpleAction) -> Self {
        use SimpleAction::*;

        match action {
            Hunt => Action::Hunt,
            Travel(d, i) => Action::Travel(d, i),
            Rest(d) => Action::Rest(d),
        }
    }
}

/// The main function that uses an Action to get a new State
///
/// # Examples
/// ```
/// root_reducer(
///   State {
///     date: Utc.ymd(2020, 3, 1),
///     miles: 2000,
///     food: 500,
///     health: 5,
///     hunt_days: 2,
///     rest_days: 2,
///     travel_days: 3,
///     travel_distance: 30,
///   },
///   Action::Travel
/// )
/// ```
fn root_reducer(state: &State, action: &Action) -> State {
    match action {
        // Travel: Move the player forward by distance and move the date forward by days
        Action::Travel(days, distance) => State {
            date: state.date + *days,
            miles: state.miles - *distance,
            ..*state
        },

        // Rest: Regenerate one health (up to 5) by stopping for rest_days
        Action::Rest(days) => State {
            date: state.date + *days,
            // No more than 5 health
            health: if state.health < 5 {
                state.health + 1
            } else {
                state.health
            },
            ..*state
        },

        // Hunt: Add one hundred pounds of food by stopping for hunt_days
        Action::Hunt => State {
            date: state.date + Duration::days(state.hunt_days),
            food: state.food + 100,
            ..*state
        },

        // Print the status of the game
        Action::Status(status_function) => status_function(*state),

        // Print commands and what they do
        Action::Help(help_function) => help_function(*state),

        // End the game
        Action::Quit(quit_mock) => quit_mock(*state),
    }
}

fn main() {
    use rand::Rng;
    use std::io;
    let inital_state = State {
        date: Utc.ymd(2020, 3, 1),
        miles: 2000,
        food: 500,
        health: 5,
        hunt_days: 2,
    };
    let mut store = Store::new(root_reducer, inital_state);

    let mut user_input = String::new();

    println!("What is your action?");

    // Store user input in user_input
    match io::stdin().read_line(&mut user_input) {
        Ok(string) => match &user_input[..] {
            "travel" => &store.dispatch(Action::Travel(
                // Random number between three and seven
                Duration::days(rand::thread_rng().gen_range(3, 7)),
                rand::thread_rng().gen_range(30, 60),
            )),
            action => &println!("Uh oh! My creator tried, but was unable to implement action {}. I've been kind of a pain.", action)
        },
        Err(error) => &println!(
            "Hmm, you put something really weird in here. The Rust language gave the error {}.",
            error
        ),
    };
}
#[cfg(test)]

mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn test_travel() {
        let initial_state = State {
            date: Utc.ymd(2020, 3, 1),
            miles: 2000,
            food: 500,
            health: 5,
            hunt_days: 2,
        };

        let result_state = State {
            miles: 1970,
            date: Utc.ymd(2020, 3, 4),
            ..initial_state
        };
        let result_state_with_more_days: State = State {
            date: Utc.ymd(2020, 3, 5),
            ..result_state
        };
        let result_state_with_more_miles: State = State {
            miles: 1960,
            ..result_state
        };

        let duration = Duration::days(3);
        let longer_duration = Duration::days(4);
        let distance = 30;
        let longer_distance = 40;

        let default_action = SimpleAction::Travel(duration, distance).into();

        assert_eq!(root_reducer(&initial_state, &default_action), result_state);

        assert_eq!(
            root_reducer(
                &initial_state,
                &SimpleAction::Travel(longer_duration, distance).into()
            ),
            result_state_with_more_days
        );
        assert_eq!(
            root_reducer(
                &initial_state,
                &SimpleAction::Travel(duration, longer_distance).into()
            ),
            result_state_with_more_miles
        );
    }

    #[test]
    fn test_rest() {
        let initial_state = State {
            date: Utc.ymd(2020, 3, 1),
            miles: 2000,
            food: 500,
            health: 4,
            hunt_days: 2,
        };
        let duration = Duration::days(2);

        assert_eq!(
            root_reducer(&initial_state, &SimpleAction::Rest(duration).into()),
            State {
                date: Utc.ymd(2020, 3, 3),
                ..initial_state
            }
        );
        assert_eq!(
            root_reducer(&initial_state, &SimpleAction::Rest(duration).into()),
            State {
                date: Utc.ymd(2020, 3, 3),
                ..initial_state
            }
        );
    }
    #[test]
    fn test_hunt() {
        let initial_state = State {
            date: Utc.ymd(2020, 3, 1),
            miles: 2000,
            food: 500,
            health: 5,
            hunt_days: 2,
        };
        let state_with_more_days: State = State {
            hunt_days: 3,
            ..initial_state
        };
        let result_state: State = State {
            date: Utc.ymd(2020, 3, 3),
            food: 600,
            ..initial_state
        };
        let result_state_with_more_days = State {
            date: Utc.ymd(2020, 3, 4),
            hunt_days: 3,
            ..result_state
        };

        assert_eq!(
            root_reducer(&initial_state, &SimpleAction::Hunt.into()),
            result_state
        );

        assert_eq!(
            root_reducer(&state_with_more_days, &SimpleAction::Hunt.into()),
            result_state_with_more_days
        );
    }

    #[test]
    fn test_status() {
        let default_state = State {
            date: Utc.ymd(2020, 3, 1),
            miles: 1970,
            food: 500,
            health: 5,
            hunt_days: 2,
        };
        let status_mock_called = Cell::new(false);

        let status_mock = |state: State| -> State {
            status_mock_called.set(true);
            return state;
        };
        assert_eq!(
            root_reducer(&default_state, &Action::Status(Box::new(status_mock))),
            default_state
        );

        assert!(status_mock_called.get());
    }

    #[test]
    fn test_help() {
        let help_mock_called = Cell::new(false);

        let help_mock = |state: State| -> State {
            help_mock_called.set(true);
            return state;
        };
        let default_state = State {
            date: Utc.ymd(2020, 3, 1),
            miles: 1970,
            food: 500,
            health: 5,
            hunt_days: 2,
        };
        root_reducer(&default_state, &Action::Help(Box::new(help_mock)));
        assert!(help_mock_called.get());
    }
    fn test_quit() {
        let quit_mock_called = Cell::new(false);

        let quit_mock = |state: State| -> State {
            quit_mock_called.set(true);
            return state;
        };
        let default_state = State {
            date: Utc.ymd(2020, 3, 1),
            miles: 1970,
            food: 500,
            health: 5,
            hunt_days: 2,
        };
        root_reducer(&default_state, &Action::Quit(Box::new(quit_mock)));
        assert!(quit_mock_called.get());
    }
}
