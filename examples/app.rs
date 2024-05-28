use bevy::{prelude::*};
use bevy_mod_sequencer::{
    action::{self, Action},
    Sequencer, SequencerPlugin,
};
use std::time::Duration;

fn action() -> impl Action<In = (), Out = ()> {
    action::animate(0., 100., Duration::from_secs(1)).map(|n| {
        dbg!(n);
    })
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SequencerPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    let mut sequencer = Sequencer::default();
    sequencer.push(action());
    commands.spawn(sequencer);
}
