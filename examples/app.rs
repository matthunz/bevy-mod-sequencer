use bevy_mod_sequencer::action::{self, Action};
use std::time::Duration;

fn action() -> impl Action {
    action::animate(0., 100., Duration::from_secs(1))
}

fn main() {}
