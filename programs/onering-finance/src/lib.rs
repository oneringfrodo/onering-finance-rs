use anchor_lang::prelude::*;

/// args
pub mod args;
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

use crate::{args::*, processor::*, traits::Processor};

declare_id!("RNGF2q87ouXMQGTxgcFPrxdUC2SFTx9HoBvhCSfpuUd");

#[program]
pub mod onering_finance {
    use super::*;

    /// initialize a main state, transaction executor is set to as an admin
    pub fn create_admin(ctx: Context<CreateAdmin>, args: CreateAdminArgs) -> ProgramResult {
        // TODO: validate main state address
        ctx.accounts.process(args)
    }

    /// apply new admin authority of main state
    pub fn apply_new_admin(ctx: Context<ApplyNewAdmin>, args: ApplyNewAdminArgs) -> ProgramResult {
        // TODO: validate main state address
        ctx.accounts.process(args)
    }

    /// update main state
    pub fn update_state(ctx: Context<UpdateState>, args: UpdateStateArgs) -> ProgramResult {
        // TODO: validate main state address
        ctx.accounts.process(args)
    }

    /// create a market for stable tokens,
    /// a stable token will have correspond pool address for the market
    pub fn create_market(ctx: Context<CreateMarket>, args: CreateMarketArgs) -> ProgramResult {
        // TODO: validate market state address
        ctx.accounts.process(args)
    }

    /// mint 1USD token in any stable tokens available
    pub fn mint(ctx: Context<Mint>, args: DepositOrWithdrawArgs) -> ProgramResult {
        // TODO: validate market state address
        ctx.accounts.process(args)
    }

    /// redeem 1USD token in USDC
    /// we initially support withdraw in USDC only, so `withdrawal_liquidity` for other markets will be 0.
    /// if we don't have enough `withdrawal_liquidity` in USDC market,
    /// we will let them wait for another one week until we add `withdrawal_liquidity` with the harvested assets from APY farms.
    pub fn redeem(ctx: Context<Redeem>, args: DepositOrWithdrawArgs) -> ProgramResult {
        // TODO: validate market state address
        ctx.accounts.process(args)
    }

    /// create a deposit reserve account
    /// it will be used to keep track of deposits and rewards
    pub fn create_reserve(ctx: Context<CreateReserve>, args: CreateReserveArgs) -> ProgramResult {
        ctx.accounts.process(args)
    }

    /// deposit (old stake) 1USD token
    pub fn deposit(ctx: Context<Deposit>, args: DepositOrWithdrawArgs) -> ProgramResult {
        ctx.accounts.process(args)
    }

    /// mint & deposit
    pub fn mint_and_deposit(ctx: Context<MintAndDeposit>, args: DepositOrWithdrawArgs) -> ProgramResult {
        // TODO: validate market state address
        ctx.accounts.process(args)
    }

    /// withdraw (old unstake) 1USD token
    pub fn withdraw(ctx: Context<Withdraw>, args: DepositOrWithdrawArgs) -> ProgramResult {
        ctx.accounts.process(args)
    }

    /// claim for accumulated 1USD in reward of deposited (old staked) 1USD tokens
    /// users can withdraw (old unstake) 1USD tokens
    pub fn claim(ctx: Context<Claim>, args: DepositOrWithdrawArgs) -> ProgramResult {
        ctx.accounts.process(args)
    }

    /// claim & deposit
    /// users claim and deposit (old stake) 1USD tokens
    pub fn claim_and_deposit(ctx: Context<Claim>, args: DepositOrWithdrawArgs) -> ProgramResult {
        ctx.accounts.process(args)
    }

    //================================================================
    // Saber Stable Swap
    //================================================================

    /// deposit to Saber stable swap pool
    pub fn saber_deposit(ctx: Context<SaberDeposit>, args: SaberDepositArgs) -> ProgramResult {
        ctx.accounts.process(args)
    }

    /// withdraw from Saber stable swap pool
    pub fn saber_withdraw(ctx: Context<SaberWithdraw>, args: SaberWithdrawArgs) -> ProgramResult {
        ctx.accounts.process(args)
    }

    /// withdraw one from Saber stable swap pool
    pub fn saber_withdraw_one(ctx: Context<SaberWithdrawOne>, args: SaberWithdrawOneArgs) -> ProgramResult {
        ctx.accounts.process(args)
    }

    //================================================================
    // Quarry Mine
    //================================================================

    /// stakes tokens into the quarry miner
    pub fn quarry_stake_tokens(ctx: Context<QuarryUserStake>, args: DepositOrWithdrawArgs) -> ProgramResult {
        ctx.accounts.process_stake_tokens(args)
    }

    /// stakes tokens into the quarry miner
    pub fn quarry_withdraw_tokens(ctx: Context<QuarryUserStake>, args: DepositOrWithdrawArgs) -> ProgramResult {
        ctx.accounts.process_withdraw_tokens(args)
    }
}
