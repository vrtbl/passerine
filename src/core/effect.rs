use std::marker::PhantomData;

use crate::vm::data::Data;

// TODO: switch from using From to TryFrom.

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EffectId(usize);

pub struct Handler<'a, T: From<&'a Data>> {
    id:    EffectId,
    _into: PhantomData<&'a T>,
}

pub struct Effect {
    pub id: EffectId,
    data:   Data,
}

impl Effect {
    #[inline(always)]
    pub fn matches<'a, T>(&'a self, handler: Handler<'a, T>) -> Option<T>
    where
        T: From<&'a Data>,
    {
        if self.id == handler.id {
            Some((&self.data).into())
        } else {
            None
        }
    }
}
