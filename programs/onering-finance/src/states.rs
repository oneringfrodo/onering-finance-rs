use anchor_lang::prelude::*;

use crate::{constant::*, located::*, traits::*};

//-----------------------------------------------------

/// onering main state
#[account]
pub struct State {
    /// admin address, privileged access to market state
    pub admin: Pubkey,

    /// 1USD mint, collateral asset
    /// Mint Address: 1USD6bWynacpTnSy2xHpSNPEDh59TYGx2VztNVzy9pZ
    pub ousd_mint: Pubkey,

    /// 1USD mint authority bump seed
    pub ousd_mint_auth_bump: u8,

    /// total deposit amount
    /// accumulated when users deposit stable tokens
    /// reducted when new withdrawal_liq is provided
    pub deposit_amount: u64,

    /// total reward amount
    /// accumulated when harvested yield is sold and converted into stable tokens
    /// reducted when reserve states are refreshed
    pub reward_amount: u64,

    /// first total reward update time, used to calculate proportional reward amount in a certain duration
    pub first_update_time: i64,

    /// last total reward update time
    pub last_update_time: i64,

    /// emergency flag
    pub emergency_flag: bool,
}

impl<T> MintAuthority for T
where
    T: Located<State>,
{
    fn with_mint_auth_seeds<R, F: FnOnce(&[&[u8]]) -> R>(&self, f: F) -> R {
        f(&[
            OUSD_MINT_AUTH_SEED,
            &self.key().as_ref(),
            &[self.as_ref().ousd_mint_auth_bump],
        ])
    }
}

//-----------------------------------------------------

/// Market state corresponds to a stable token
#[account]
pub struct Market {
    /// stable mint; USDC, USDt, etc.,
    pub stable_mint: Pubkey,

    /// stable vault authority bump seed
    pub stable_vault_bump: u8,

    /// withdrawal liquid
    pub withdrawal_liq: u64,

    /// lock flag
    pub lock_flag: bool,
}

impl<T> VaultAuthority for T
where
    T: Located<Market>,
{
    fn with_vault_auth_seeds<R, F: FnOnce(&[&[u8]]) -> R>(&self, f: F) -> R {
        f(&[
            &self.as_ref().stable_mint.key().as_ref(),
            STABLE_VAULT_SEED,
            &self.key().as_ref(),
            &[self.as_ref().stable_vault_bump],
        ])
    }
}

//-----------------------------------------------------

/// Reserved account
#[account]
pub struct Reserve {
    /// accumulated deposit amount of stable tokens no matter of its type
    pub deposit_amount: u64,

    /// harvested yield amount
    pub reward_amount: u64,

    /// last update time, unix timestamp
    pub last_update_time: i64,

    /// freeze flag, disable reserved account in case of emergency
    pub freeze_flag: bool,
}

impl Reserve {
    /// refresh reserve state
    pub fn refresh_reserve(&mut self, state: &mut State) {
        let elapsed_time =
            if state.last_update_time > self.last_update_time && self.last_update_time > 0 {
                state.last_update_time.saturating_sub(self.last_update_time)
            } else {
                0
            };

        // proportional amount of deposit
        let deposit_prop = self.deposit_amount / state.deposit_amount;
        // proportional duration
        let duration_prop = elapsed_time / (state.last_update_time - state.first_update_time);
        // proportional reward
        let reward_prop = state.reward_amount * deposit_prop * ((duration_prop as u32) as u64);

        // update reward amount
        self.reward_amount += reward_prop;

        // update total reward amount
        state.reward_amount -= reward_prop;

        // update last updated time
        self.last_update_time = state.last_update_time;
    }
}

//-----------------------------------------------------
