use anchor_lang::{prelude::*, solana_program::clock};
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount, Transfer};
use std::mem::size_of;

use crate::{args::*, constant::*, error::*, states::*, traits::*};

//-----------------------------------------------------

/// accounts for [create_reserve]
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
            RESERVE_SEED.as_ref(),
            state.key().as_ref(),
        ],
        bump = args.nonce,
        payer = initializer,
        space = 8 + size_of::<Reserve>(),
    )]
    pub reserve: Box<Account<'info, Reserve>>,

    /// main state
    #[account(
        mut,
        constraint = !state.emergency_flag @ OneRingFinanceError::ServiceDisabled,
    )]
    pub state: Box<Account<'info, State>>,

    /// system program
    pub system_program: Program<'info, System>,
}

/// implementation for [CreateReserve]
impl<'info> CreateReserve<'info> {
    /// process [create_reserve]
    pub fn process(&mut self, args: CreateReserveArgs) -> ProgramResult {
        self.reserve.nonce = args.nonce;
        self.reserve.deposit_amount = 0;
        self.reserve.reward_amount = 0;
        self.reserve.last_update_time = 0;
        self.reserve.freeze_flag = false;

        Ok(())
    }
}

//-----------------------------------------------------

/// accounts for [deposit]
#[derive(Accounts)]
#[instruction(args: DepositOrWithdrawArgs)]
pub struct Deposit<'info> {
    /// user, deposit initializer
    pub initializer: Signer<'info>,

    /// 1USD mint, collateral asset
    #[account(
        mut,
        constraint = ousd_mint.key().eq(&state.ousd_mint) @ OneRingFinanceError::InvalidOusdMint,
    )]
    pub ousd_mint: Box<Account<'info, Mint>>,

    /// 1USD token
    #[account(
        mut,
        constraint = initializer_ousd_token.owner.eq(initializer.key) @ OneRingFinanceError::InvalidOusdAccountOwner,
        constraint = initializer_ousd_token.mint.eq(&ousd_mint.key()) @ OneRingFinanceError::InvalidOusdMint,
        constraint = initializer_ousd_token.amount >= args.amount @ OneRingFinanceError::InsufficientOusdBalance,
    )]
    pub initializer_ousd_token: Box<Account<'info, TokenAccount>>,

    /// reserve state
    #[account(
        mut,
        seeds = [
            initializer.key().as_ref(),
            RESERVE_SEED.as_ref(),
            state.key().as_ref(),
        ],
        bump = reserve.nonce,
        constraint = !reserve.freeze_flag @ OneRingFinanceError::ReserveFrozen,
    )]
    pub reserve: Box<Account<'info, Reserve>>,

    /// main state
    #[account(
        mut,
        constraint = !state.emergency_flag @ OneRingFinanceError::ServiceDisabled,
    )]
    pub state: Box<Account<'info, State>>,

    /// token program
    pub token_program: Program<'info, Token>,
}

/// implementation for [Deposit]
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

    /// process [deposit]
    /// deposit 1USD for reward (old stake)
    pub fn process(&mut self, args: DepositOrWithdrawArgs) -> ProgramResult {
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

/// accounts for [mint_and_deposit]
#[derive(Accounts)]
#[instruction(args: DepositOrWithdrawArgs)]
pub struct MintAndDeposit<'info> {
    /// user, reserve initializer
    pub initializer: Signer<'info>,

    /// stable mint
    #[account(
        constraint = stable_mint.key().eq(&market.stable_mint) @ OneRingFinanceError::InvalidStableMint,
    )]
    pub stable_mint: Box<Account<'info, Mint>>,

    /// stable vault
    #[account(
        mut,
        seeds = [
            market.stable_mint.key().as_ref(),
            STABLE_VAULT_SEED.as_ref(),
            market.key().as_ref()
        ],
        bump = market.stable_vault_bump,
        constraint = stable_vault.mint.eq(&stable_mint.key()) @ OneRingFinanceError::InvalidStableMint,
    )]
    pub stable_vault: Box<Account<'info, TokenAccount>>,

    /// stable token
    #[account(
        mut,
        constraint = initializer_stable_token.owner.eq(initializer.key) @ OneRingFinanceError::InvalidStableAccountOwner,
        constraint = initializer_stable_token.mint.eq(&stable_mint.key()) @ OneRingFinanceError::InvalidStableMint,
        constraint = initializer_stable_token.amount >= args.amount @ OneRingFinanceError::InsufficientStableBalance,
    )]
    pub initializer_stable_token: Box<Account<'info, TokenAccount>>,

    /// 1USD mint, collateral asset
    #[account(
        mut,
        constraint = ousd_mint.key().eq(&state.ousd_mint) @ OneRingFinanceError::InvalidOusdMint,
    )]
    pub ousd_mint: Box<Account<'info, Mint>>,

    /// reserve state
    #[account(
        mut,
        seeds = [
            initializer.key().as_ref(),
            RESERVE_SEED.as_ref(),
            state.key().as_ref(),
        ],
        bump = reserve.nonce,
        constraint = !reserve.freeze_flag @ OneRingFinanceError::ReserveFrozen,
    )]
    pub reserve: Box<Account<'info, Reserve>>,

    /// market state
    #[account(
        constraint = !market.lock_flag @ OneRingFinanceError::MarketLocked,
    )]
    pub market: Box<Account<'info, Market>>,

    /// main state
    #[account(
        mut,
        constraint = !state.emergency_flag @ OneRingFinanceError::ServiceDisabled,
    )]
    pub state: Box<Account<'info, State>>,

    /// token program
    pub token_program: Program<'info, Token>,
}

/// implementation for [MintAndDeposit]
impl<'info> MintAndDeposit<'info> {
    /// transfer stable token from initializer to vault
    pub fn transfer_to_vault(&self, amount: u64) -> ProgramResult {
        let cpi_accounts = Transfer {
            from: self.initializer_stable_token.to_account_info(),
            to: self.stable_vault.to_account_info(),
            authority: self.initializer.to_account_info(),
        };

        token::transfer(
            CpiContext::new(self.token_program.to_account_info(), cpi_accounts),
            amount,
        )
    }

    /// process [deposit_and_mint]
    /// deposit 1USD directly for reward (old stake)
    /// no actual mint needed
    pub fn process(&mut self, args: DepositOrWithdrawArgs) -> ProgramResult {
        // transfer stable token from initializer to vault
        self.transfer_to_vault(args.amount)?;

        // $1USD amount equivalant to stable token amount
        let ousd_amount = if self.stable_mint.decimals > self.ousd_mint.decimals {
            args.amount
                / u64::pow(
                    10,
                    (self.stable_mint.decimals - self.ousd_mint.decimals) as u32,
                )
        } else if self.stable_mint.decimals < self.ousd_mint.decimals {
            args.amount
                * u64::pow(
                    10,
                    (self.ousd_mint.decimals - self.stable_mint.decimals) as u32,
                )
        } else {
            args.amount
        };

        // initialize first update time
        if self.state.first_update_time == 0 {
            self.state.first_update_time = clock::Clock::get().unwrap().unix_timestamp;
        }

        // refresh reserve state
        self.reserve.refresh_reserve(&mut self.state);

        // accumulate deposit amount of any stable tokens
        self.reserve.deposit_amount += ousd_amount;

        // add stake liquidity, used to calculate rewards
        self.state.deposit_amount += ousd_amount;

        Ok(())
    }
}

//-----------------------------------------------------

/// accounts for [withdraw]
#[derive(Accounts)]
#[instruction(args: DepositOrWithdrawArgs)]
pub struct Withdraw<'info> {
    /// user, reserve initializer
    pub initializer: Signer<'info>,

    /// 1USD mint, collateral asset
    #[account(
        mut,
        constraint = ousd_mint.key().eq(&state.ousd_mint) @ OneRingFinanceError::InvalidOusdMint,
    )]
    pub ousd_mint: Box<Account<'info, Mint>>,

    /// 1USD mint authority
    pub ousd_mint_auth: UncheckedAccount<'info>,

    /// 1USD token
    #[account(
        mut,
        constraint = initializer_ousd_token.owner.eq(initializer.key) @ OneRingFinanceError::InvalidOusdAccountOwner,
        constraint = initializer_ousd_token.mint.eq(&ousd_mint.key()) @ OneRingFinanceError::InvalidOusdMint,
    )]
    pub initializer_ousd_token: Box<Account<'info, TokenAccount>>,

    /// reserve state
    #[account(
        mut,
        seeds = [
            initializer.key().as_ref(),
            RESERVE_SEED.as_ref(),
            state.key().as_ref(),
        ],
        bump = reserve.nonce,
        constraint = !reserve.freeze_flag @ OneRingFinanceError::ReserveFrozen,
        constraint = reserve.deposit_amount >= args.amount @ OneRingFinanceError::WithdrawalAmountTooMuch,
    )]
    pub reserve: Box<Account<'info, Reserve>>,

    /// main state
    #[account(
        mut,
        constraint = !state.emergency_flag @ OneRingFinanceError::ServiceDisabled,
    )]
    pub state: Box<Account<'info, State>>,

    /// token program
    pub token_program: Program<'info, Token>,
}

/// implementatin for [withdraw]
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

    /// process [withdraw]
    /// widthdraw, burn same amount of 1USD
    pub fn process(&mut self, args: DepositOrWithdrawArgs) -> ProgramResult {
        // refresh reserve state
        self.reserve.refresh_reserve(&mut self.state);

        // reduct deposit amount
        self.reserve.deposit_amount -= args.amount;

        // reduct stake liquidity
        self.state.deposit_amount -= args.amount;

        // mint withdraw amount of 1USD to initializer
        self.mint_to_initializer(args.amount)?;

        Ok(())
    }
}

//-----------------------------------------------------

/// accounts for [claim]
#[derive(Accounts)]
#[instruction(args: DepositOrWithdrawArgs)]
pub struct Claim<'info> {
    /// user, reserve initializer
    pub initializer: Signer<'info>,

    /// 1USD mint, collateral asset
    #[account(
        constraint = ousd_mint.key().eq(&state.ousd_mint) @ OneRingFinanceError::InvalidOusdMint,
    )]
    pub ousd_mint: Box<Account<'info, Mint>>,

    /// 1USD mint authority
    pub ousd_mint_auth: UncheckedAccount<'info>,

    /// 1USD token
    #[account(
        mut,
        constraint = initializer_ousd_token.owner.eq(initializer.key) @ OneRingFinanceError::InvalidOusdAccountOwner,
        constraint = initializer_ousd_token.mint.eq(&ousd_mint.key()) @ OneRingFinanceError::InvalidOusdMint,
    )]
    pub initializer_ousd_token: Box<Account<'info, TokenAccount>>,

    /// reserve state
    #[account(
        mut,
        seeds = [
            initializer.key().as_ref(),
            RESERVE_SEED.as_ref(),
            state.key().as_ref(),
        ],
        bump = reserve.nonce,
        constraint = !reserve.freeze_flag @ OneRingFinanceError::ReserveFrozen,
    )]
    pub reserve: Box<Account<'info, Reserve>>,

    /// main state
    #[account(
        mut,
        constraint = !state.emergency_flag @ OneRingFinanceError::ServiceDisabled,
    )]
    pub state: Box<Account<'info, State>>,

    /// token program
    pub token_program: Program<'info, Token>,
}

/// implementation for [claim]
impl<'info> Claim<'info> {
    /// mint claim amount of 1USD to initializer
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

    /// process [claim]
    /// claim for rewards
    pub fn process(&mut self, args: DepositOrWithdrawArgs) -> ProgramResult {
        // refresh reserve state
        self.reserve.refresh_reserve(&mut self.state);

        // check if claim amount less than reward amount
        if self.reserve.reward_amount < args.amount {
            return Err(OneRingFinanceError::ClaimAmountTooMuch.into());
        }

        // reduct reward amount
        self.reserve.reward_amount -= args.amount;

        // mint claim amount of 1USD to initializer
        self.mint_to_initializer(args.amount)?;

        Ok(())
    }
}

//-----------------------------------------------------

/// accounts for [claim_and_deposit]
#[derive(Accounts)]
#[instruction(args: DepositOrWithdrawArgs)]
pub struct ClaimAndDeposit<'info> {
    /// user, reserve initializer
    pub initializer: Signer<'info>,

    /// reserve state
    #[account(
        mut,
        seeds = [
            initializer.key().as_ref(),
            RESERVE_SEED.as_ref(),
            state.key().as_ref(),
        ],
        bump = reserve.nonce,
        constraint = !reserve.freeze_flag @ OneRingFinanceError::ReserveFrozen,
    )]
    pub reserve: Box<Account<'info, Reserve>>,

    /// main state
    #[account(
        mut,
        constraint = !state.emergency_flag @ OneRingFinanceError::ServiceDisabled,
    )]
    pub state: Box<Account<'info, State>>,

    /// token program
    pub token_program: Program<'info, Token>,
}

/// implementation for [ClaimAndDeposit]
impl<'info> ClaimAndDeposit<'info> {
    /// process [cliam_and_deposit]
    /// claim and deposit directly, transfer or burn not needed
    pub fn process(&mut self, args: DepositOrWithdrawArgs) -> ProgramResult {
        // refresh reserve state
        self.reserve.refresh_reserve(&mut self.state);

        // check if claim amount less than reward amount
        if self.reserve.reward_amount < args.amount {
            return Err(OneRingFinanceError::ClaimAmountTooMuch.into());
        }

        // reduct reward amount
        self.reserve.reward_amount -= args.amount;

        // accumulate deposit amount of any stable tokens
        self.reserve.deposit_amount += args.amount;

        // add stake liquidity, used to calculate rewards
        self.state.deposit_amount += args.amount;

        Ok(())
    }
}

//-----------------------------------------------------
