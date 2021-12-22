use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{constant::*, error::*, states::*, traits::*};

//-----------------------------------------------------

/// accounts for market initialization
#[derive(Accounts)]
#[instruction(args: CreateMarketArgs)]
pub struct CreateMarket<'info> {
    /// admin, market initializer
    #[account(mut)]
    pub admin: Signer<'info>,

    /// stable mint
    pub stable_mint: Box<Account<'info, Mint>>,

    /// stable vault
    #[account(
        init,
        seeds = [
            stable_mint.key().as_ref(),
            STABLE_VAULT_SEED,
            market.key().as_ref()
        ],
        bump = args.stable_vault_bump,
        payer = admin,
        token::mint = stable_mint,
        token::authority = stable_vault,
    )]
    pub stable_vault: Box<Account<'info, TokenAccount>>,

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

/// initialize market corresponds to a stable token
impl<'info> Processor<CreateMarketArgs> for CreateMarket<'info> {
    fn process(&mut self, args: CreateMarketArgs) -> ProgramResult {
        self.market.stable_mint = self.stable_mint.key();
        self.market.stable_vault_bump = args.stable_vault_bump;

        self.market.withdrawal_liq = 0;

        self.market.lock_flag = false;

        Ok(())
    }
}

//-----------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct CreateMarketArgs {
    pub stable_vault_bump: u8,
}
