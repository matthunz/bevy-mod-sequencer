use super::Action;
use bevy::ecs::system::{ParamSet, SystemParamItem};
use std::task::Poll;

pub struct AndThen<A, F, B> {
    a: A,
    f: F,
    next: Option<B>,
}

impl<A, F, B> AndThen<A, F, B> {
    pub(crate) fn new(a: A, f: F) -> Self {
        Self { a, f, next: None }
    }
}

impl<A, F, B> Action for AndThen<A, F, B>
where
    A: Action,
    F: FnMut(A::Out) -> B,
    B: Action<In = A::In>,
{
    type In = A::In;

    type Params = ParamSet<'static, 'static, (A::Params, B::Params)>;

    type Out = B::Out;

    fn perform(
        &mut self,
        input: Self::In,
        mut params: SystemParamItem<Self::Params>,
    ) -> Poll<Option<Self::Out>> {
        if let Some(ref mut next) = self.next {
            match next.perform(input, params.p1()) {
                Poll::Ready(Some(out)) => Poll::Ready(Some(out)),
                Poll::Ready(None) => {
                    self.next = None;
                    Poll::Pending
                }
                poll => poll,
            }
        } else {
            match self.a.perform(input, params.p0()) {
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
