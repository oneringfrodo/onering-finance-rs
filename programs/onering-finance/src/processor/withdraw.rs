use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

use crate::{error::*, states::*, traits::*};

//-----------------------------------------------------

/// accounts for withdraw
#[derive(Accounts)]
#[instruction(args: WithdrawArgs)]
pub struct Withdraw<'info> {
    /// user, reserve initializer
    #[account()]
    pub initializer: Signer<'info>,

    /// stable mint
    #[account(
        constraint = stable_mint.key().eq(&market.stable_mint) @ CommonError::InvalidStableMint,
    )]
    pub stable_mint: Box<Account<'info, Mint>>,

    /// stable vault
    #[account(
        mut,
        constraint = stable_vault.mint.eq(&stable_mint.key()) @ CommonError::InvalidStableMint,
    )]
    pub stable_vault: Box<Account<'info, TokenAccount>>,

    /// stable token
    #[account(
        mut,
        constraint = initializer_stable_token.owner.eq(initializer.key) @ CommonError::InvalidStableAccountOwner,
        constraint = initializer_stable_token.mint.eq(&stable_mint.key()) @ CommonError::InvalidStableMint,
    )]
    pub initializer_stable_token: Box<Account<'info, TokenAccount>>,

    /// 1USD mint, collateral asset
    #[account(
        constraint = ousd_mint.key().eq(&state.ousd_mint) @ CommonError::InvalidOusdMint,
    )]
    pub ousd_mint: Box<Account<'info, Mint>>,

    /// 1USD token
    #[account(
        mut,
        constraint = initializer_ousd_token.owner.eq(initializer.key) @ CommonError::InvalidOusdAccountOwner,
        constraint = initializer_ousd_token.mint.eq(&ousd_mint.key()) @ CommonError::InvalidOusdMint,
        constraint = initializer_ousd_token.amount >= args.amount @ CommonError::InsufficientOusdBalance,
    )]
    pub initializer_ousd_token: Box<Account<'info, TokenAccount>>,

    /// reserve state
    #[account(
        mut,
        constraint = !reserve.freeze_flag @ CommonError::ReserveFrozen,
        constraint = reserve.deposit_amount >= args.amount @ CommonError::WithdrawalAmountTooMuch,
    )]
    pub reserve: Box<Account<'info, Reserve>>,

    /// market state
    #[account(
        mut,
        constraint = !market.lock_flag @ CommonError::MarketLocked,
        constraint = market.withdrawal_liq >= args.amount @ CommonError::InsufficientWithdrawalLiquidity,
    )]
    pub market: Box<Account<'info, Market>>,

    /// main state
    #[account(
        mut,
        constraint = !state.emergency_flag @ CommonError::ServiceDisabled,
    )]
    pub state: Box<Account<'info, State>>,

    /// token program
    pub token_program: Program<'info, Token>,
}

impl<'info> Withdraw<'info> {
    /// transfer stable token from vault to initializer
    pub fn transfer_to_initializer(&self, amount: u64) -> ProgramResult {
        let cpi_accounts = Transfer {
            from: self.stable_vault.to_account_info(),
            to: self.initializer_stable_token.to_account_info(),
            authority: self.stable_vault.to_account_info(),
        };

        self.market.with_vault_auth_seeds(|mint_seeds| {
            token::transfer(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(),
                    cpi_accounts,
                    &[mint_seeds],
                ),
                amount,
            )
        })
    }

    /// burn withdrawal amount of 1USD from initializer
    pub fn burn_from_initializer(&self, amount: u64) -> ProgramResult {
        let cpi_accounts = Burn {
            mint: self.ousd_mint.to_account_info(),
            to: self.initializer_ousd_token.to_account_info(),
            authority: self.initializer.to_account_info(),
        };

        token::burn(
            CpiContext::new(self.token_program.to_account_info(), cpi_accounts),
            amount,
        )
    }
}

/// widthdraw, burn same amount of 1USD
impl<'info> Processor<WithdrawArgs> for Withdraw<'info> {
    fn process(&mut self, args: WithdrawArgs) -> ProgramResult {
        // refresh reserve state
        self.reserve.refresh_reserve(&mut self.state);

        // reduct deposit amount
        self.reserve.deposit_amount -= args.amount;

        // reduct withdrawal liquid
        self.market.withdrawal_liq -= args.amount;

        // transfer stable token from vault to initializer
        self.transfer_to_initializer(args.amount)?;

        // burn withdrawal amount of 1USD from initializer
        self.burn_from_initializer(args.amount)?;

        Ok(())
    }
}

//-----------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawArgs {
    pub amount: u64,
}
