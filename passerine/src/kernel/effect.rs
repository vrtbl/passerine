use std::marker::PhantomData;

use crate::{
    vm::data::Data,
    Inject,
};

// TODO: switch from using From to TryFrom.

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EffectId(usize);

pub struct Handler<T: Inject> {
    id:    EffectId,
    _into: PhantomData<T>,
}

pub struct Effect {
    pub id:         EffectId,
    unmatched_data: Option<Data>,
}

impl Effect {
    #[inline(always)]
    pub fn matches<T>(&mut self, handler: Handler<T>) -> Option<Result<T, ()>>
    where
        T: Inject,
    {
        if self.id == handler.id {
            let data = std::mem::replace(&mut self.unmatched_data, None);
            data.map(TryInto::try_into)
        } else {
            None
        }
    }
}
