use anchor_lang::prelude::*;
use anchor_spl::token::{Mint as TokenMint, Token, TokenAccount};

use crate::{args::*, constant::*, error::*, states::*, traits::*};

//-----------------------------------------------------

/// accounts for [create_market]
#[derive(Accounts)]
#[instruction(args: CreateMarketArgs)]
pub struct CreateMarket<'info> {
    /// admin, market initializer
    #[account(mut)]
    pub admin: Signer<'info>,

    /// stable mint
    pub stable_mint: Box<Account<'info, TokenMint>>,

    /// stable vault
    #[account(
        init,
        seeds = [
            stable_mint.key().as_ref(),
            STABLE_VAULT_SEED.as_ref(),
            market.key().as_ref()
        ],
        bump = args.stable_vault_bump,
        payer = admin,
        token::mint = stable_mint,
        token::authority = stable_vault_auth,
    )]
    pub stable_vault: Box<Account<'info, TokenAccount>>,

    /// stable vault authority
    #[account(
        seeds = [
            STABLE_VAULT_SEED.as_ref(),
            state.key().as_ref()
        ],
        bump = state.stable_vault_auth_bump,
    )]
    pub stable_vault_auth: UncheckedAccount<'info>,

    /// market state
    #[account(zero)]
    pub market: Box<Account<'info, Market>>,

    /// main state
    #[account(
        has_one = admin @ CommonError::AccessDenied,
    )]
    pub state: Box<Account<'info, State>>,

    /// token program
    pub token_program: Program<'info, Token>,

    /// system program
    pub system_program: Program<'info, System>,

    /// rent var
    pub rent: Sysvar<'info, Rent>,
}

/// process [create_market]
impl<'info> Processor<CreateMarketArgs> for CreateMarket<'info> {
    /// initialize market corresponds to a stable token
    fn process(&mut self, args: CreateMarketArgs) -> ProgramResult {
        self.market.stable_mint = self.stable_mint.key();
        self.market.stable_vault_bump = args.stable_vault_bump;

        self.market.withdrawal_liq = 0;

        self.market.lock_flag = false;

        Ok(())
    }
}

//-----------------------------------------------------
