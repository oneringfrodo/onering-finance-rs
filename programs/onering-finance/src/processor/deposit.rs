use anchor_lang::{prelude::*, solana_program::clock};
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount};
use std::mem::size_of;

use crate::{constant::*, error::*, states::*, traits::*};

//-----------------------------------------------------

/// accounts for reserve
#[derive(Accounts)]
#[instruction(args: CreateReserveArgs)]
pub struct CreateReserve<'info> {
    /// user, reserve initializer
    #[account(mut)]
    pub initializer: Signer<'info>,

    /// reserve state
    #[account(
        init,
        seeds = [
            initializer.key().as_ref(),
            RESERVE_SEED,
            state.key().as_ref(),
        ],
        bump = args.reserve_bump.unwrap(),
        payer = initializer,
        space = 8 + size_of::<Reserve>(),
    )]
    pub reserve: Box<Account<'info, Reserve>>,

    /// market state
    #[account(
        mut,
        constraint = !market.lock_flag @ CommonError::MarketLocked,
    )]
    pub market: Box<Account<'info, Market>>,

    /// main state
    #[account(
        mut,
        constraint = !state.emergency_flag @ CommonError::ServiceDisabled,
    )]
    pub state: Box<Account<'info, State>>,

    /// system program
    pub system_program: Program<'info, System>,

    /// rent var
    pub rent: Sysvar<'info, Rent>,
}

/// create reserve
impl<'info> Processor<CreateReserveArgs> for CreateReserve<'info> {
    fn process(&mut self, _args: CreateReserveArgs) -> ProgramResult {
        self.reserve.deposit_amount = 0;
        self.reserve.reward_amount = 0;
        self.reserve.last_update_time = 0;
        self.reserve.freeze_flag = false;

        Ok(())
    }
}

//-----------------------------------------------------

/// accounts for reserve
#[derive(Accounts)]
#[instruction(args: DepositOrWithdrawArgs)]
pub struct Deposit<'info> {
    /// user, reserve initializer
    pub initializer: Signer<'info>,

    /// 1USD mint, collateral asset
    #[account(
        mut,
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
    )]
    pub reserve: Box<Account<'info, Reserve>>,

    /// main state
    #[account(
        mut,
        constraint = !state.emergency_flag @ CommonError::ServiceDisabled,
    )]
    pub state: Box<Account<'info, State>>,

    /// token program
    pub token_program: Program<'info, Token>,
}

impl<'info> Deposit<'info> {
    /// burn deposit (old stake) amount of 1USD from initializer
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

/// deposit 1USD for reward (old stake)
impl<'info> Processor<DepositOrWithdrawArgs> for Deposit<'info> {
    fn process(&mut self, args: DepositOrWithdrawArgs) -> ProgramResult {
        // initialize first update time
        if self.state.first_update_time == 0 {
            self.state.first_update_time = clock::Clock::get().unwrap().unix_timestamp;
        }

        // refresh reserve state
        self.reserve.refresh_reserve(&mut self.state);

        // accumulate deposit amount of any stable tokens
        self.reserve.deposit_amount += args.amount;

        // add stake liquidity, used to calculate rewards
        self.state.deposit_amount += args.amount;

        // mint deposit amount of 1USD to initializer
        self.burn_from_initializer(args.amount)?;

        Ok(())
    }
}

//-----------------------------------------------------

/// accounts for withdraw
#[derive(Accounts)]
#[instruction(args: DepositOrWithdrawArgs)]
pub struct Withdraw<'info> {
    /// user, reserve initializer
    pub initializer: Signer<'info>,

    /// 1USD mint, collateral asset
    #[account(
        constraint = ousd_mint.key().eq(&state.ousd_mint) @ CommonError::InvalidOusdMint,
    )]
    pub ousd_mint: Box<Account<'info, Mint>>,

    /// 1USD mint authority
    pub ousd_mint_auth: UncheckedAccount<'info>,

    /// 1USD token
    #[account(
        mut,
        constraint = initializer_ousd_token.owner.eq(initializer.key) @ CommonError::InvalidOusdAccountOwner,
        constraint = initializer_ousd_token.mint.eq(&ousd_mint.key()) @ CommonError::InvalidOusdMint,
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
    /// mint withdraw amount of 1USD to initializer
    pub fn mint_to_initializer(&self, amount: u64) -> ProgramResult {
        let cpi_accounts = MintTo {
            mint: self.ousd_mint.to_account_info(),
            to: self.initializer_ousd_token.to_account_info(),
            authority: self.ousd_mint_auth.to_account_info(),
        };

        self.state.with_mint_auth_seeds(|mint_seeds| {
            token::mint_to(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(),
                    cpi_accounts,
                    &[mint_seeds],
                ),
                amount,
            )
        })
    }
}

/// widthdraw, burn same amount of 1USD
impl<'info> Processor<DepositOrWithdrawArgs> for Withdraw<'info> {
    fn process(&mut self, args: DepositOrWithdrawArgs) -> ProgramResult {
        // refresh reserve state
        self.reserve.refresh_reserve(&mut self.state);

        // reduct deposit amount
        self.reserve.deposit_amount -= args.amount;

        // reduct withdrawal liquid
        self.market.withdrawal_liq -= args.amount;

        // reduct stake liquidity
        self.state.deposit_amount -= args.amount;

        // mint withdraw amount of 1USD to initializer
        self.mint_to_initializer(args.amount)?;

        Ok(())
    }
}

//-----------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct CreateReserveArgs {
    pub reserve_bump: Option<u8>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct DepositOrWithdrawArgs {
    pub amount: u64,
}
