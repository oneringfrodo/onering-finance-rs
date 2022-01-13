use anchor_lang::prelude::*;
use anchor_spl::token::Token;

use crate::{args::*, error::*, states::*, traits::*};

//-----------------------------------------------------

/// accounts for [PortDeposit]
#[derive(Accounts, Clone)]
pub struct PortDeposit<'info> {
    /// admin
    #[account(mut)]
    pub admin: Signer<'info>,

    /// main state
    #[account(
        has_one = admin @ CommonError::AccessDenied,
        constraint = !state.emergency_flag @ CommonError::ServiceDisabled,
    )]
    pub state: Box<Account<'info, State>>,

    /// port finance program
    pub port_finance_program: UncheckedAccount<'info>,

    // cpi accounts below
    #[account(mut)]
    pub source_liquidity: UncheckedAccount<'info>,
    #[account(mut)]
    pub destination_collateral: UncheckedAccount<'info>,
    #[account(mut)]
    pub reserve: UncheckedAccount<'info>,
    #[account(mut)]
    pub reserve_liquidity_supply: UncheckedAccount<'info>,
    #[account(mut)]
    pub reserve_collateral_mint: UncheckedAccount<'info>,
    #[account(mut)]
    pub lending_market: UncheckedAccount<'info>,
    pub lending_market_authority: UncheckedAccount<'info>,
    pub transfer_authority: UncheckedAccount<'info>,
    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
}

/// implementation for [PortDeposit]
impl<'info> PortDeposit<'info> {
    /// process [deposit]
    pub fn process(&self, args: DepositOrWithdrawArgs) -> ProgramResult {
        let cpi_accounts = port_anchor_adaptor::Deposit {
            source_liquidity: self.source_liquidity.to_account_info(),
            destination_collateral: self.destination_collateral.to_account_info(),
            reserve: self.reserve.to_account_info(),
            reserve_liquidity_supply: self.reserve_liquidity_supply.to_account_info(),
            reserve_collateral_mint: self.reserve_collateral_mint.to_account_info(),
            lending_market: self.lending_market.to_account_info(),
            lending_market_authority: self.lending_market_authority.to_account_info(),
            transfer_authority: self.transfer_authority.to_account_info(),
            clock: self.clock.to_account_info(),
            token_program: self.token_program.to_account_info(),
        };

        let cpi_context =
            CpiContext::new(self.port_finance_program.to_account_info(), cpi_accounts);

        self.state.with_vault_auth_seeds(|auth_seeds| {
            port_anchor_adaptor::deposit_reserve(
                cpi_context.with_signer(&[auth_seeds]),
                args.amount,
            )
        })
    }
}

//-----------------------------------------------------

/// accounts for [port_withdraw]
#[derive(Accounts)]
pub struct PortWithdraw<'info> {
    /// admin
    #[account(mut)]
    pub admin: Signer<'info>,

    /// main state
    #[account(
        has_one = admin @ CommonError::AccessDenied,
        constraint = !state.emergency_flag @ CommonError::ServiceDisabled,
    )]
    pub state: Box<Account<'info, State>>,

    /// port finance program
    pub port_finance_program: UncheckedAccount<'info>,

    // cpi accounts below
    #[account(mut)]
    pub source_collateral: UncheckedAccount<'info>,
    #[account(mut)]
    pub destination_collateral: UncheckedAccount<'info>,
    #[account(mut)]
    pub reserve: UncheckedAccount<'info>,
    #[account(mut)]
    pub obligation: UncheckedAccount<'info>,
    #[account(mut)]
    pub lending_market: UncheckedAccount<'info>,
    pub lending_market_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub stake_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub staking_pool: UncheckedAccount<'info>,
    pub obligation_owner: UncheckedAccount<'info>,
    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub port_staking_program: UncheckedAccount<'info>,
}

/// implementation for [PortWithdraw]
impl<'info> PortWithdraw<'info> {
    /// process [withdraw]
    pub fn process(&self, args: DepositOrWithdrawArgs) -> ProgramResult {
        let cpi_accounts = port_anchor_adaptor::Withdraw {
            source_collateral: self.source_collateral.to_account_info(),
            destination_collateral: self.destination_collateral.to_account_info(),
            reserve: self.reserve.to_account_info(),
            obligation: self.obligation.to_account_info(),
            lending_market: self.lending_market.to_account_info(),
            lending_market_authority: self.lending_market_authority.to_account_info(),
            stake_account: self.stake_account.to_account_info(),
            staking_pool: self.staking_pool.to_account_info(),
            obligation_owner: self.obligation_owner.to_account_info(),
            clock: self.clock.to_account_info(),
            token_program: self.token_program.to_account_info(),
            port_staking_program: self.port_staking_program.to_account_info(),
        };

        let cpi_context =
            CpiContext::new(self.port_finance_program.to_account_info(), cpi_accounts);

        port_anchor_adaptor::withdraw(cpi_context, args.amount)
    }
}

//-----------------------------------------------------

#[derive(Accounts)]
pub struct PortClaimReward<'info> {
    /// admin
    #[account(mut)]
    pub admin: Signer<'info>,

    /// main state
    #[account(
        has_one = admin @ CommonError::AccessDenied,
        constraint = !state.emergency_flag @ CommonError::ServiceDisabled,
    )]
    pub state: Box<Account<'info, State>>,

    /// port finance program
    pub port_finance_program: UncheckedAccount<'info>,

    // cpi accounts below
    #[account(mut)]
    pub stake_account_owner: UncheckedAccount<'info>,
    #[account(mut)]
    pub stake_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub staking_pool: UncheckedAccount<'info>,
    #[account(mut)]
    pub reward_token_pool: UncheckedAccount<'info>,
    #[account(mut)]
    pub reward_dest: UncheckedAccount<'info>,
    pub staking_program_authority: UncheckedAccount<'info>,
    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
}

/// implementation for [PortClaimReward]
impl<'info> PortClaimReward<'info> {
    /// process [claim_reward]
    pub fn process(&self) -> ProgramResult {
        let cpi_accounts = port_anchor_adaptor::ClaimReward {
            stake_account_owner: self.stake_account_owner.to_account_info(),
            stake_account: self.stake_account.to_account_info(),
            staking_pool: self.staking_pool.to_account_info(),
            reward_token_pool: self.reward_token_pool.to_account_info(),
            reward_dest: self.reward_dest.to_account_info(),
            staking_program_authority: self.staking_program_authority.to_account_info(),
            clock: self.clock.to_account_info(),
            token_program: self.token_program.to_account_info(),
        };

        let cpi_context =
            CpiContext::new(self.port_finance_program.to_account_info(), cpi_accounts);

        port_anchor_adaptor::claim_reward(cpi_context)
    }
}

//-----------------------------------------------------
