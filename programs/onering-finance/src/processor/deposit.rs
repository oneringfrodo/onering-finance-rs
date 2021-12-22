use anchor_lang::{prelude::*, solana_program::clock};
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};
use std::mem::size_of;

use crate::{constant::*, error::*, states::*, traits::*};

//-----------------------------------------------------

/// accounts for reserve
#[derive(Accounts)]
#[instruction(args: DepositArgs)]
pub struct CreateReserve<'info> {
    /// user, reserve initializer
    #[account(mut)]
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
        constraint = initializer_stable_token.amount >= args.amount @ CommonError::InsufficientStableBalance,
    )]
    pub initializer_stable_token: Box<Account<'info, TokenAccount>>,

    /// 1USD mint, collateral asset
    #[account(
        mut,
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

    /// token program
    pub token_program: Program<'info, Token>,

    /// system program
    pub system_program: Program<'info, System>,

    /// rent var
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> CreateReserve<'info> {
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

    /// mint deposit amount of 1USD to initializer
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

/// create reserve
impl<'info> Processor<DepositArgs> for CreateReserve<'info> {
    fn process(&mut self, args: DepositArgs) -> ProgramResult {
        self.reserve.deposit_amount = args.amount;
        self.reserve.last_update_time = 0;
        self.reserve.freeze_flag = false;

        // add liquidity, used to calculate rewards
        self.state.deposit_amount += args.amount;

        // transfer stable token from initializer to vault
        self.transfer_to_vault(args.amount)?;

        // mint deposit amount of 1USD to initializer
        self.mint_to_initializer(args.amount)?;

        // initialize first update time
        if self.state.first_update_time == 0 {
            self.state.first_update_time = clock::Clock::get().unwrap().unix_timestamp;
        }

        // refresh reserve state
        self.reserve.refresh_reserve(&mut self.state);

        Ok(())
    }
}

//-----------------------------------------------------

/// accounts for reserve
#[derive(Accounts)]
#[instruction(args: DepositArgs)]
pub struct Deposit<'info> {
    /// user, reserve initializer
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
        constraint = initializer_stable_token.amount >= args.amount @ CommonError::InsufficientStableBalance,
    )]
    pub initializer_stable_token: Box<Account<'info, TokenAccount>>,

    /// 1USD mint, collateral asset
    #[account(
        mut,
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

    /// token program
    pub token_program: Program<'info, Token>,
}

impl<'info> Deposit<'info> {
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

    /// mint deposit amount of 1USD to initializer
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

/// deposit to the market
impl<'info> Processor<DepositArgs> for Deposit<'info> {
    fn process(&mut self, args: DepositArgs) -> ProgramResult {
        // refresh reserve state
        self.reserve.refresh_reserve(&mut self.state);

        // accumulate deposit amount of any stable tokens
        self.reserve.deposit_amount += args.amount;

        // add liquidity, used to calculate rewards
        self.state.deposit_amount += args.amount;

        // transfer stable token from initializer to vault
        self.transfer_to_vault(args.amount)?;

        // mint deposit amount of 1USD to initializer
        self.mint_to_initializer(args.amount)?;

        Ok(())
    }
}

//-----------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct DepositArgs {
    pub amount: u64,
    pub reserve_bump: Option<u8>,
}
