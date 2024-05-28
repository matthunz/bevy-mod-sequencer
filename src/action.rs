use bevy::{
    ecs::system::{SystemParam, SystemParamItem},
    prelude::*,
};
use std::{marker::PhantomData, time::Duration};

pub trait Action {
    type In;

    type Params: SystemParam;

    type Out;

    fn perform(
        &mut self,
        input: Self::In,
        params: SystemParamItem<Self::Params>,
    ) -> Option<Self::Out>;

    fn map<F, R>(self, f: F) -> Map<Self, F, R>
    where
        Self: Sized,
        F: FnMut(Self::Out) -> R,
    {
        Map {
            action: self,
            f ,
            _marker: PhantomData,
        }
    }
}

pub fn from_fn<F, Marker>(f: F) -> FromFn<F, Marker>
where
    F: SystemParamFunction<Marker>,
{
    FromFn {
        f,
        _marker: PhantomData,
    }
}

pub struct FromFn<F, Marker> {
    f: F,
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
    ) -> Option<Self::Out> {
        Some(self.f.run(input, params))
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
        input: Self::In,
        params: SystemParamItem<Self::Params>,
    ) -> Option<Self::Out> {
        let start = *self.start.get_or_insert_with(|| params.elapsed_seconds());
        let elapsed = params.elapsed_seconds() - start;

        if elapsed < self.duration.as_secs_f32() {
            let t = elapsed / self.duration.as_secs_f32();
            Some(T::interpolate(&self.from, &self.to, t))
        } else {
            None
        }
    }
}

pub struct Map<A, F, R> {
    action: A,
    f: F,
    _marker: PhantomData<R>,
}

impl<A, F, R> Action for Map<A, F, R>
where
    A: Action,
    F: FnMut(A::Out) -> R,
{
    type In = A::In;

    type Params = A::Params;

    type Out = R;

    fn perform(
        &mut self,
        input: Self::In,
        params: SystemParamItem<Self::Params>,
    ) -> Option<Self::Out> {
        self.action.perform(input, params).map(|out| (self.f)(out))
    }
}