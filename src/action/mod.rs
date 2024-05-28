use bevy::{
    ecs::system::{SystemParam, SystemParamItem, SystemState},
    prelude::*,
};
use std::{marker::PhantomData, task::Poll, time::Duration};

mod map;
pub use self::map::Map;
pub trait Action {
    type In;

    type Params: SystemParam;

    type Out;

    fn perform(
        &mut self,
        input: Self::In,
        params: SystemParamItem<Self::Params>,
    ) -> Poll<Option<Self::Out>>;

    fn map<F, B>(self, f: F) -> Map<Self, Self::Out, F, B>
    where
        Self: Sized,
        F: FnMut(Self::Out) -> B,
        B: Action<In = ()>,
    {
        Map::new(self, f)
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
        to,
        start: None,
        duration,
    }
}

pub struct Animate<T> {
    from: T,
    to: T,
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
        let start = *self.start.get_or_insert_with(|| params.elapsed_seconds());
        let elapsed = params.elapsed_seconds() - start;

        if elapsed < self.duration.as_secs_f32() {
            let t = elapsed / self.duration.as_secs_f32();
            Poll::Ready(Some(T::interpolate(&self.from, &self.to, t)))
        } else {
            Poll::Ready(None)
        }
    }
}
