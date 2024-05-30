# bevy-mod-sequencer

Action sequence graph plugin for turn-based games.

```rust
use bevy::prelude::*;
use bevy_mod_sequencer::{
    action::{self, Action},
    Sequencer, SequencerPlugin,
};
use std::time::Duration;

#[derive(Component)]
struct Slider;

fn action() -> impl Action<In = (), Out = ()> {
    action::animate(0f32, 400., Duration::from_secs(10))
        .map(|n| action::from_fn(move || n * 2.))
        .map(|n| {
            action::from_fn(move |mut query: Query<&mut Transform, With<Slider>>| {
                for mut transform in &mut query {
                    transform.translation.x = n;
                }
            })
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

    commands.spawn(Camera2dBundle::default());

    for _ in 0..100 {
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
}
```
