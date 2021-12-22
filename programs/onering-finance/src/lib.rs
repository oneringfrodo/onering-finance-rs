use anchor_lang::prelude::*;

/// constant
pub mod constant;
/// error
pub mod error;
/// located
pub mod located;
/// processor
pub mod processor;
/// states
pub mod states;
/// traits
pub mod traits;

use processor::*;
use traits::Processor;

declare_id!("oRnG11SQmnUr8QM3QS351ZG1dQcgNtvZhrMZCRA4CNf");

#[program]
pub mod onering_finance {
    use super::*;

    /// initialize a main state, transaction executor is set to as an admin
    pub fn create_admin(ctx: Context<CreateAdmin>, args: CreateAdminArgs) -> ProgramResult {
        ctx.accounts.process(args)
    }

    /// change admin authority of main state
    pub fn update_admin(ctx: Context<UpdateAdmin>, args: UpdateAdminArgs) -> ProgramResult {
        ctx.accounts.process(args)
    }

    /// update main state
    pub fn update_state(ctx: Context<UpdateState>, args: UpdateStateArgs) -> ProgramResult {
        ctx.accounts.process(args)
    }

    /// create a market for stable tokens,
    /// a stable token will have correspond pool address for the market
    pub fn create_market(ctx: Context<CreateMarket>, args: CreateMarketArgs) -> ProgramResult {
        ctx.accounts.process(args)
    }

    /// create a reserve when users deposit stable tokens
    /// it will be used to keep track of deposits and rewards
    pub fn create_reserve(ctx: Context<CreateReserve>, args: DepositArgs) -> ProgramResult {
        ctx.accounts.process(args)
    }

    /// deposit stable token to a market
    pub fn deposit(ctx: Context<Deposit>, args: DepositArgs) -> ProgramResult {
        ctx.accounts.process(args)
    }

    /// withdraw stable token from USDC market
    /// we initially support withdraw in USDC only, so `withdrawal_liquidity` for other markets will be 0.
    /// if we don't have enough `withdrawal_liquidity` in USDC market,
    /// we will let them wait for another one week until we add `withdrawal_liquidity` with the harvested assets from APY farms.
    pub fn withdraw(ctx: Context<Withdraw>, args: WithdrawArgs) -> ProgramResult {
        ctx.accounts.process(args)
    }

    /// claim for accumulated 1USD in reward of deposited stable tokens
    /// users can withdraw anothre USDC using this 1USD tokens
    pub fn claim(ctx: Context<Claim>, args: ClaimArgs) -> ProgramResult {
        ctx.accounts.process(args)
    }
}
