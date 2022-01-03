use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint as TokenMint, MintTo, Token, TokenAccount, Transfer};

use crate::{args::*, error::*, states::*, traits::*};

//-----------------------------------------------------

/// accounts for mint
#[derive(Accounts)]
#[instruction(args: DepositOrWithdrawArgs)]
pub struct Mint<'info> {
    /// user, mint initializer
    pub initializer: Signer<'info>,

    /// stable mint
    #[account(
        constraint = stable_mint.key().eq(&market.stable_mint) @ CommonError::InvalidStableMint,
    )]
    pub stable_mint: Box<Account<'info, TokenMint>>,

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
    pub ousd_mint: Box<Account<'info, TokenMint>>,

    /// 1USD mint authority
    pub ousd_mint_auth: UncheckedAccount<'info>,

    /// 1USD token
    #[account(
        mut,
        constraint = initializer_ousd_token.owner.eq(initializer.key) @ CommonError::InvalidOusdAccountOwner,
        constraint = initializer_ousd_token.mint.eq(&ousd_mint.key()) @ CommonError::InvalidOusdMint,
    )]
    pub initializer_ousd_token: Box<Account<'info, TokenAccount>>,

    /// market state
    #[account(
        constraint = !market.lock_flag @ CommonError::MarketLocked,
    )]
    pub market: Box<Account<'info, Market>>,

    /// main state
    #[account(
        constraint = !state.emergency_flag @ CommonError::ServiceDisabled,
    )]
    pub state: Box<Account<'info, State>>,

    /// token program
    pub token_program: Program<'info, Token>,
}

impl<'info> Mint<'info> {
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
impl<'info> Processor<DepositOrWithdrawArgs> for Mint<'info> {
    fn process(&mut self, args: DepositOrWithdrawArgs) -> ProgramResult {
        // transfer stable token from initializer to vault
        self.transfer_to_vault(args.amount)?;

        // mint deposit amount of 1USD to initializer
        self.mint_to_initializer(args.amount)?;

        Ok(())
    }
}

//-----------------------------------------------------

/// accounts for redeem
#[derive(Accounts)]
#[instruction(args: DepositOrWithdrawArgs)]
pub struct Redeem<'info> {
    /// user, redeem initializer
    pub initializer: Signer<'info>,

    /// stable mint
    #[account(
        constraint = stable_mint.key().eq(&market.stable_mint) @ CommonError::InvalidStableMint,
    )]
    pub stable_mint: Box<Account<'info, TokenMint>>,

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
    pub ousd_mint: Box<Account<'info, TokenMint>>,

    /// 1USD token
    #[account(
        mut,
        constraint = initializer_ousd_token.owner.eq(initializer.key) @ CommonError::InvalidOusdAccountOwner,
        constraint = initializer_ousd_token.mint.eq(&ousd_mint.key()) @ CommonError::InvalidOusdMint,
        constraint = initializer_ousd_token.amount >= args.amount @ CommonError::InsufficientOusdBalance,
    )]
    pub initializer_ousd_token: Box<Account<'info, TokenAccount>>,

    /// market state
    #[account(
        constraint = !market.lock_flag @ CommonError::MarketLocked,
        constraint = market.withdrawal_liq >= args.amount @ CommonError::InsufficientWithdrawalLiquidity,
    )]
    pub market: Box<Account<'info, Market>>,

    /// main state
    #[account(
        constraint = !state.emergency_flag @ CommonError::ServiceDisabled,
    )]
    pub state: Box<Account<'info, State>>,

    /// token program
    pub token_program: Program<'info, Token>,
}

impl<'info> Redeem<'info> {
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

    /// burn redeem amount of 1USD from initializer
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

/// redeem, burn same amount of 1USD
impl<'info> Processor<DepositOrWithdrawArgs> for Redeem<'info> {
    fn process(&mut self, args: DepositOrWithdrawArgs) -> ProgramResult {
        // transfer stable token from vault to initializer
        self.transfer_to_initializer(args.amount)?;

        // burn redeem amount of 1USD from initializer
        self.burn_from_initializer(args.amount)?;

        // reduct withdrawal liquid
        self.market.withdrawal_liq -= args.amount;

        Ok(())
    }
}

//-----------------------------------------------------