use std::ops::DerefMut;

use anchor_lang::prelude::*;

/// parsed account with location key
pub trait Located<T> {
    fn as_ref(&self) -> &T;
    fn as_mut(&mut self) -> &mut T;
    fn key(&self) -> Pubkey;
}

impl<'info, T, A> Located<T> for A
where
    A: ToAccountInfo<'info> + DerefMut<Target = T>,
{
    fn as_ref(&self) -> &T {
        self.deref()
    }

    fn as_mut(&mut self) -> &mut T {
        self.deref_mut()
    }

    fn key(&self) -> Pubkey {
        *self.to_account_info().key
    }
}
