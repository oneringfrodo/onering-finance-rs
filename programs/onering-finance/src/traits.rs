use anchor_lang::prelude::*;

pub trait Processor<T> {
    fn process(&mut self, args: T) -> ProgramResult;
}

pub trait MintAuthority {
    fn with_mint_auth_seeds<R, F: FnOnce(&[&[u8]]) -> R>(&self, f: F) -> R;
}

pub trait VaultAuthority {
    fn with_vault_auth_seeds<R, F: FnOnce(&[&[u8]]) -> R>(&self, f: F) -> R;
}
