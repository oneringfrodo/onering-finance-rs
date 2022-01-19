use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::{args::*, error::*, states::*};

//-----------------------------------------------------

/// accounts for [create_admin]
#[derive(Accounts)]
pub struct CreateAdmin<'info> {
    /// admin, onering initializer
    pub admin: Signer<'info>,

    /// 1USD mint, collateral asset
    pub ousd_mint: Box<Account<'info, Mint>>,

    /// main state
    #[account(zero)]
    pub state: Box<Account<'info, State>>,
}

/// implementation for [CreateAdmin]
impl<'info> CreateAdmin<'info> {
    /// process [create_admin]
    pub fn process(&mut self, args: CreateAdminArgs) -> ProgramResult {
        self.state.admin = self.admin.key();

        self.state.ousd_mint = self.ousd_mint.key();
        self.state.ousd_mint_auth_bump = args.ousd_mint_auth_bump;

        self.state.stable_vault_auth_bump = args.stable_vault_auth_bump;

        self.state.deposit_amount = 0;
        self.state.reward_amount = 0;
        self.state.first_update_time = 0;
        self.state.last_update_time = 0;

        self.state.emergency_flag = false;

        Ok(())
    }
}

//-----------------------------------------------------

/// accounts for [apply_new_admin]
#[derive(Accounts)]
pub struct ApplyNewAdmin<'info> {
    /// admin
    pub admin: Signer<'info>,

    /// new admin
    pub new_admin: UncheckedAccount<'info>,

    /// main state
    #[account(
        mut,
        has_one = admin @ CommonError::AccessDenied,
    )]
    pub state: Box<Account<'info, State>>,
}

/// implementation for [ApplyNewAdmin]
impl<'info> ApplyNewAdmin<'info> {
    /// process [apply_new_admin]
    pub fn process(&mut self) -> ProgramResult {
        self.state.admin = self.new_admin.key();

        Ok(())
    }
}

//-----------------------------------------------------

/// accounts for [update_state]
#[derive(Accounts)]
pub struct UpdateState<'info> {
    /// admin, onering initializer
    pub admin: Signer<'info>,

    /// global state
    #[account(
        mut,
        has_one = admin @ CommonError::AccessDenied,
    )]
    pub state: Box<Account<'info, State>>,
}

/// implementation for [UpdateState]
impl<'info> UpdateState<'info> {
    /// process [update_state]
    pub fn process(&mut self, args: UpdateStateArgs) -> ProgramResult {
        self.state.emergency_flag = args.emergency_flag;

        Ok(())
    }
}

//-----------------------------------------------------
