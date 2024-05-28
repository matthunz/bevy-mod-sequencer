use action::{Action, AnyAction};
use bevy::{ecs::system::SystemId, prelude::*};
use std::collections::VecDeque;

pub mod action;

pub struct SequencerPlugin;

impl Plugin for SequencerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, run_sequencer);
    }
}

#[derive(Default, Component)]
pub struct Sequencer {
    actions: VecDeque<Box<dyn AnyAction<In = (), Out = ()>>>,
    pending: VecDeque<SystemId<(), ()>>,
}

impl Sequencer {
    pub fn push(&mut self, action: impl Action<In = (), Out = ()> + Send + Sync + 'static) {
        self.actions.push_back(Box::new(action));
    }
}

fn run_sequencer(world: &mut World) {
    let mut new = Vec::new();
    let mut pending = Vec::new();

    for (entity, mut sequencer) in world.query::<(Entity, &mut Sequencer)>().iter_mut(world) {
        if let Some(action) = sequencer.actions.pop_front() {
            new.push((entity, action));
        }

        pending.extend(sequencer.pending.iter().cloned());
    }

    for (entity, mut action) in new {
        let id = world.register_system(move |world: &mut World| {
            if action.perform_any((), world).is_some() {
                let mut sequencer = world.get_mut::<Sequencer>(entity).unwrap();
                sequencer.pending.pop_front();
            }
        });

        let mut sequencer = world.get_mut::<Sequencer>(entity).unwrap();
        sequencer.pending.push_back(id);
        pending.push(id);
    }

    for id in pending {
        world.run_system(id).unwrap();
    }
}
