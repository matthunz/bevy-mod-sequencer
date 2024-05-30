use super::Action;
use bevy::ecs::system::{ParamSet, SystemParamItem};
use std::task::Poll;

pub struct Then<A, B> {
    a: A,
    b: B,
    is_next: bool,
}

impl<A, B> Then<A, B> {
    pub(crate) fn new(a: A, b: B) -> Self {
        Self {
            a,
            b,
            is_next: false,
        }
    }
}

impl<A, B> Action for Then<A, B>
where
    A: Action,
    B: Action<In = A::In, Out = A::Out>,
{
    type In = A::In;

    type Params = ParamSet<'static, 'static, (A::Params, B::Params)>;

    type Out = A::Out;

    fn perform(
        &mut self,
        input: Self::In,
        mut params: SystemParamItem<Self::Params>,
    ) -> Poll<Option<Self::Out>> {
        if self.is_next {
            match self.b.perform(input, params.p1()) {
                Poll::Ready(Some(out)) => Poll::Ready(Some(out)),
                poll => poll,
            }
        } else {
            match self.a.perform(input, params.p0()) {
                Poll::Ready(Some(out)) => Poll::Ready(Some(out)),
                Poll::Ready(None) => {
                    self.is_next = true;
                    Poll::Pending
                }
                Poll::Pending => Poll::Pending,
            }
        }
    }
}
