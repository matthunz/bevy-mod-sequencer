use bevy::{ecs::system::RunSystemOnce, prelude::*};
use bevy_mod_sequencer::action::{self, Action};
use std::time::Duration;

fn action() -> impl Action<In = (), Out = ()> {
    action::animate(0., 100., Duration::from_secs(1)).map(|n| {
        dbg!(n);
    })
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup(action()))
        .run();
}

fn setup<A>(action: A) -> impl FnMut(&mut World)
where
    A: Action<In = (), Out = ()> + Send + Sync + 'static,
    A::Params: 'static,
{
    let mut action_cell = Some(action);
    move |world| {
        let mut action = action_cell.take().unwrap();
        world.run_system_once(move |mut params: ParamSet<(A::Params,)>| {
            action.perform((), params.p0());
        });
    }
}
