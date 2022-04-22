use std::marker::PhantomData;

use crate::{
    vm::data::Data,
    Inject,
};

// TODO: switch from using From to TryFrom.

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EffectId(usize);

pub struct Handler<'a, T: Inject<'a>> {
    id:    EffectId,
    _into: PhantomData<&'a T>,
}

pub struct Effect {
    pub id: EffectId,
    data:   Data,
}

impl Effect {
    #[inline(always)]
    pub fn matches<'a, T>(
        &'a self,
        handler: Handler<'a, T>,
    ) -> Option<Result<T, ()>>
    where
        T: Inject<'a>,
    {
        if self.id == handler.id {
            Some((&self.data).try_into())
        } else {
            None
        }
    }
}
