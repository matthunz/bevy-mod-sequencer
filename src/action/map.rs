use super::Action;
use bevy::ecs::system::SystemParamItem;
use std::{marker::PhantomData, task::Poll};

pub struct Map<A, O, F, B> {
    action: A,
    f: F,
    next: Option<B>,
    _marker: PhantomData<fn(O)>,
}

impl<A, O, F, B> Map<A, O, F, B> {
    pub(super) fn new(action: A, f: F) -> Self {
        Self {
            action,
            f,
            next: None,
            _marker: PhantomData,
        }
    }
}

impl<A, O, F, B> Action for Map<A, O, F, B>
where
    A: Action<Out = O>,
    F: FnMut(O) -> B,
    B: Action<In = ()>,
{
    type In = A::In;

    type Params = (A::Params, B::Params);

    type Out = B::Out;

    fn perform(
        &mut self,
        input: Self::In,
        params: SystemParamItem<Self::Params>,
    ) -> Poll<Option<Self::Out>> {
        if let Some(ref mut next) = self.next {
            match next.perform((), params.1) {
                Poll::Ready(None) => {
                    self.next = None;

                    match self.action.perform(input, params.0) {
                        Poll::Ready(Some(out)) => {
                            self.next = Some((self.f)(out));
                            Poll::Pending
                        }
                        Poll::Ready(None) => Poll::Ready(None),
                        Poll::Pending => Poll::Pending,
                    }
                }
                poll => poll,
            }
        } else {
            match self.action.perform(input, params.0) {
                Poll::Ready(Some(out)) => {
                    self.next = Some((self.f)(out));
                    Poll::Pending
                }
                Poll::Ready(None) => Poll::Ready(None),
                Poll::Pending => Poll::Pending,
            }
        }
    }
}
