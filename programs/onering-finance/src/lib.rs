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

    pub fn create_admin(ctx: Context<CreateAdmin>, args: CreateAdminArgs) -> ProgramResult {
        ctx.accounts.process(args)
    }

    pub fn update_admin(ctx: Context<UpdateAdmin>, args: UpdateAdminArgs) -> ProgramResult {
        ctx.accounts.process(args)
    }

    pub fn update_state(ctx: Context<UpdateState>, args: UpdateStateArgs) -> ProgramResult {
        ctx.accounts.process(args)
    }

    pub fn create_market(ctx: Context<CreateMarket>, args: CreateMarketArgs) -> ProgramResult {
        ctx.accounts.process(args)
    }

    pub fn create_reserve(ctx: Context<CreateReserve>, args: DepositArgs) -> ProgramResult {
        ctx.accounts.process(args)
    }

    pub fn deposit(ctx: Context<Deposit>, args: DepositArgs) -> ProgramResult {
        ctx.accounts.process(args)
    }

    pub fn withdraw(ctx: Context<Withdraw>, args: WithdrawArgs) -> ProgramResult {
        ctx.accounts.process(args)
    }

    pub fn claim(ctx: Context<Claim>, args: ClaimArgs) -> ProgramResult {
        ctx.accounts.process(args)
    }
}
