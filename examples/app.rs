use bevy::prelude::*;
use bevy_mod_sequencer::{
    action::{self, Action},
    Sequencer, SequencerPlugin,
};
use std::time::Duration;

#[derive(Component)]
struct Slider;

fn action() -> impl Action<In = (), Out = ()> {
    action::animate(0., 100., Duration::from_secs(1)).for_each(action::from_fn(
        |In(x), mut query: Query<&mut Transform, With<Slider>>| {
            for mut transform in &mut query {
                transform.translation.x = x;
            }
        },
    ))
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

    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.25, 0.25, 0.75),
                custom_size: Some(Vec2::new(50.0, 100.0)),
                ..default()
            },
            ..default()
        },
        Slider,
    ));
}
