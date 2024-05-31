use and_then::AndThen;
use bevy::{
    ecs::{
        system::{SystemParam, SystemParamItem, SystemState},
        world::unsafe_world_cell::UnsafeWorldCell,
    },
    prelude::*,
};
use std::{any::Any, marker::PhantomData, task::Poll, time::Duration};

mod and_then;

mod map;
pub use self::map::Map;

mod then;
pub use self::then::Then;

pub trait Action {
    type In;

    type Params: SystemParam;

    type Out;

    fn perform(
        &mut self,
        input: Self::In,
        params: SystemParamItem<Self::Params>,
    ) -> Poll<Option<Self::Out>>;

    fn and_then<F, A>(self, f: F) -> AndThen<Self, F, A>
    where
        Self: Sized,
        F: FnMut(Self::Out) -> A,
        A: Action<In = Self::In>,
    {
        AndThen::new(self, f)
    }

    fn map<F, B>(self, f: F) -> Map<Self, Self::Out, F, B>
    where
        Self: Sized,
        F: FnMut(Self::Out) -> B,
        B: Action<In = ()>,
    {
        Map::new(self, f)
    }

    fn then<A>(self, action: A) -> Then<Self, A>
    where
        Self: Sized,
        A: Action<In = Self::In>,
    {
        Then::new(self, action)
    }
}

pub struct DynParam<'w> {
    world: UnsafeWorldCell<'w>,
}

unsafe impl SystemParam for DynParam<'_> {
    type State = ();

    type Item<'world, 'state> = DynParam<'world>;

    fn init_state(
        _world: &mut World,
        _system_meta: &mut bevy::ecs::system::SystemMeta,
    ) -> Self::State {
    }

    unsafe fn get_param<'world, 'state>(
        _state: &'state mut Self::State,
        _system_meta: &bevy::ecs::system::SystemMeta,
        world: UnsafeWorldCell<'world>,
        _change_tick: bevy::ecs::component::Tick,
    ) -> Self::Item<'world, 'state> {
        DynParam { world }
    }
}

impl<I, O> Action for Box<dyn AnyAction<In = I, Out = O>> {
    type In = I;

    type Params = DynParam<'static>;

    type Out = O;

    fn perform(
        &mut self,
        input: Self::In,
        params: SystemParamItem<Self::Params>,
    ) -> Poll<Option<Self::Out>> {
        (&mut **self).perform_any(input, unsafe { params.world.world_mut() })
    }
}

pub trait AnyAction: Send + Sync {
    type In;

    type Out;

    fn perform_any(&mut self, input: Self::In, world: &mut World) -> Poll<Option<Self::Out>>;
}

impl<A> AnyAction for A
where
    A: Action + Send + Sync,
    A::Params: 'static,
{
    type In = A::In;

    type Out = A::Out;

    fn perform_any(&mut self, input: Self::In, world: &mut World) -> Poll<Option<Self::Out>> {
        let mut state = SystemState::<A::Params>::new(world);
        let poll = self.perform(input, state.get_mut(world));
        state.apply(world);
        poll
    }
}

pub fn from_fn<F, Marker>(f: F) -> FromFn<F, Marker>
where
    F: SystemParamFunction<Marker>,
{
    FromFn {
        f,
        is_done: false,
        _marker: PhantomData,
    }
}

pub struct FromFn<F, Marker> {
    f: F,
    is_done: bool,
    _marker: PhantomData<Marker>,
}

impl<F, Marker> Action for FromFn<F, Marker>
where
    F: SystemParamFunction<Marker>,
{
    type In = F::In;

    type Params = F::Param;

    type Out = F::Out;

    fn perform(
        &mut self,
        input: Self::In,
        params: SystemParamItem<Self::Params>,
    ) -> Poll<Option<Self::Out>> {
        if self.is_done {
            Poll::Ready(None)
        } else {
            self.is_done = true;
            Poll::Ready(Some(self.f.run(input, params)))
        }
    }
}

pub fn animate<T>(from: T, to: T, duration: Duration) -> Animate<T> {
    Animate {
        from,
        to: Some(to),
        start: None,
        duration,
    }
}

pub struct Animate<T> {
    from: T,
    to: Option<T>,
    start: Option<f32>,
    duration: Duration,
}

impl<T: Animatable> Action for Animate<T> {
    type In = ();

    type Params = Res<'static, Time>;

    type Out = T;

    fn perform(
        &mut self,
        _input: Self::In,
        params: SystemParamItem<Self::Params>,
    ) -> Poll<Option<Self::Out>> {
        if let Some(ref to) = self.to {
            let start = *self.start.get_or_insert_with(|| params.elapsed_seconds());
            let elapsed = params.elapsed_seconds() - start;

            if elapsed < self.duration.as_secs_f32() {
                let t = elapsed / self.duration.as_secs_f32();
                Poll::Ready(Some(T::interpolate(&self.from, to, t)))
            } else {
                Poll::Ready(Some(self.to.take().unwrap()))
            }
        } else {
            Poll::Ready(None)
        }
    }
}

pub fn from_iter<A>(iter: impl IntoIterator<Item = A>) -> FromIter<A> {
    FromIter {
        actions: iter.into_iter().map(Some).collect(),
        idx: 0,
    }
}

pub struct FromIter<A> {
    actions: Vec<Option<A>>,
    idx: usize,
}

impl<A> Action for FromIter<A>
where
    A: Action<Out = ()>,
{
    type In = A::In;

    type Params = A::Params;

    type Out = ();

    fn perform(
        &mut self,
        input: Self::In,
        params: SystemParamItem<Self::Params>,
    ) -> Poll<Option<Self::Out>> {
        let mut idx = self.idx;
        let mut is_done = true;

        while let Some(cell) = self.actions.get_mut(idx) {
            if let Some(action) = cell {
                match action.perform(input, params) {
                    Poll::Ready(None) => *cell = None,
                    _ => {}
                }

                self.idx = if idx + 1 >= self.actions.len() {
                    0
                } else {
                    idx + 1
                };

                is_done = false;
                break;
            }

            idx += 1;
        }

        if is_done {
            Poll::Ready(None)
        } else {
            if idx + 1 >= self.actions.len() {
                self.idx = 0;
                Poll::Ready(Some(()))
            } else {
                self.idx = idx + 1;
                Poll::Pending
            }
        }
    }
}
