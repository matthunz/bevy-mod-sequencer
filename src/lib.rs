use action::{Action, AnyAction};
use bevy::{ecs::system::SystemId, prelude::*};
use std::{collections::VecDeque, task::Poll};

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
        self.push_boxed(Box::new(action));
    }

    pub fn push_boxed(&mut self, action: Box<dyn AnyAction<In = (), Out = ()>>) {
        self.actions.push_back(action);
    }
}

fn run_sequencer(world: &mut World) {
    let mut new = Vec::new();
    let mut pending = Vec::new();

    for (entity, mut sequencer) in world.query::<(Entity, &mut Sequencer)>().iter_mut(world) {
        while let Some(action) = sequencer.actions.pop_front() {
            new.push((entity, action));
        }

        pending.extend(sequencer.pending.front().cloned());
    }

    for (entity, mut action) in new {
        let id = world.register_system(move |world: &mut World| loop {
            match action.perform_any((), world) {
                Poll::Ready(Some(())) => break,
                Poll::Ready(None) => {
                    let mut sequencer = world.get_mut::<Sequencer>(entity).unwrap();
                    sequencer.pending.pop_front();
                    break;
                }
                Poll::Pending => {},
            }
        });

        let mut sequencer = world.get_mut::<Sequencer>(entity).unwrap();
        sequencer.pending.push_back(id);
    }

    for id in pending {
        world.run_system(id).unwrap();
    }
}
