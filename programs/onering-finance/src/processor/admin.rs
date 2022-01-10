use anchor_lang::prelude::*;
use anchor_spl::token::Mint as TokenMint;

use crate::{args::*, error::*, states::*, traits::Processor};

//-----------------------------------------------------

/// accounts for [create_admin]
#[derive(Accounts)]
pub struct CreateAdmin<'info> {
    /// admin, onering initializer
    pub admin: Signer<'info>,

    /// 1USD mint, collateral asset
    pub ousd_mint: Box<Account<'info, TokenMint>>,

    /// onering main state
    #[account(zero)]
    pub state: Box<Account<'info, State>>,
}

/// process [create_admin]
impl<'info> Processor<CreateAdminArgs> for CreateAdmin<'info> {
    fn process(&mut self, args: CreateAdminArgs) -> ProgramResult {
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

    /// onering main state
    #[account(
        mut,
        has_one = admin @ CommonError::AccessDenied,
    )]
    pub state: Box<Account<'info, State>>,
}

/// process [apply_new_admin]
impl<'info> Processor<ApplyNewAdminArgs> for ApplyNewAdmin<'info> {
    fn process(&mut self, _args: ApplyNewAdminArgs) -> ProgramResult {
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

/// process [update_state]
impl<'info> Processor<UpdateStateArgs> for UpdateState<'info> {
    fn process(&mut self, args: UpdateStateArgs) -> ProgramResult {
        self.state.emergency_flag = args.emergency_flag;

        Ok(())
    }
}

//-----------------------------------------------------
