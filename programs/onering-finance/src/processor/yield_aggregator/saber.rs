use anchor_lang::prelude::*;
use anchor_spl::token::Token;

use crate::{error::*, states::*, traits::*};

//-----------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct SaberDepositArgs {
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    pub min_mint_amount: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct SaberWithdrawArgs {
    pool_token_amount: u64,
    minimum_token_a_amount: u64,
    minimum_token_b_amount: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct SaberWithdrawOneArgs {
    pool_token_amount: u64,
    minimum_token_amount: u64,
}

//-----------------------------------------------------

/// accounts for [saber_deposit]
#[derive(Accounts)]
pub struct SaberDeposit<'info> {
    /// admin
    #[account(mut)]
    pub admin: Signer<'info>,

    /// main state
    #[account(
        has_one = admin @ OneRingFinanceError::AccessDenied,
        constraint = !state.emergency_flag @ OneRingFinanceError::ServiceDisabled,
    )]
    pub state: Box<Account<'info, State>>,

    /// saber stable swap program
    pub saber_program: UncheckedAccount<'info>,

    // cpi accounts below
    /// The swap.
    pub swap: UncheckedAccount<'info>,
    /// The authority of the swap.
    pub swap_authority: UncheckedAccount<'info>,
    /// The authority of the user.
    #[account(mut)]
    pub user_authority: UncheckedAccount<'info>,
    /// The token account associated with the user.
    #[account(mut)]
    pub input_a_user: UncheckedAccount<'info>,
    /// The token account for the pool's reserves of this token.
    #[account(mut)]
    pub input_a_reserve: UncheckedAccount<'info>,
    /// The token account associated with the user.
    #[account(mut)]
    pub input_b_user: UncheckedAccount<'info>,
    /// The token account for the pool's reserves of this token.
    #[account(mut)]
    pub input_b_reserve: UncheckedAccount<'info>,
    /// The pool mint of the swap.
    #[account(mut)]
    pub pool_mint: UncheckedAccount<'info>,
    /// The output account for LP tokens.
    #[account(mut)]
    pub output_lp: UncheckedAccount<'info>,
    /// The spl_token program.
    pub token_program: Program<'info, Token>,
    /// The clock
    pub clock: Sysvar<'info, Clock>,
}

/// process [saber_deposit]
impl<'info> SaberDeposit<'info> {
    /// deposit to Saber stable swap pool
    pub fn process(&self, args: SaberDepositArgs) -> ProgramResult {
        let cpi_accounts = stable_swap_anchor::Deposit {
            user: stable_swap_anchor::SwapUserContext {
                token_program: self.token_program.to_account_info(),
                swap_authority: self.swap_authority.to_account_info(),
                user_authority: self.user_authority.to_account_info(),
                swap: self.swap.to_account_info(),
                clock: self.clock.to_account_info(),
            },
            input_a: stable_swap_anchor::SwapToken {
                user: self.input_a_user.to_account_info(),
                reserve: self.input_a_reserve.to_account_info(),
            },
            input_b: stable_swap_anchor::SwapToken {
                user: self.input_b_user.to_account_info(),
                reserve: self.input_b_reserve.to_account_info(),
            },
            pool_mint: self.pool_mint.to_account_info(),
            output_lp: self.output_lp.to_account_info(),
        };

        let cpi_context = CpiContext::new(self.saber_program.to_account_info(), cpi_accounts);

        self.state.with_vault_auth_seeds(|auth_seeds| {
            stable_swap_anchor::deposit(
                cpi_context.with_signer(&[auth_seeds]),
                args.token_a_amount,
                args.token_b_amount,
                args.min_mint_amount,
            )
        })
    }
}

//-----------------------------------------------------

/// accounts for [saber_withdraw]
#[derive(Accounts)]
pub struct SaberWithdraw<'info> {
    /// admin
    #[account(mut)]
    pub admin: Signer<'info>,

    /// main state
    #[account(
        has_one = admin @ OneRingFinanceError::AccessDenied,
        constraint = !state.emergency_flag @ OneRingFinanceError::ServiceDisabled,
    )]
    pub state: Box<Account<'info, State>>,

    /// saber stable swap program
    pub saber_program: UncheckedAccount<'info>,

    // cpi accounts below
    /// The swap.
    pub swap: UncheckedAccount<'info>,
    /// The authority of the swap.
    pub swap_authority: UncheckedAccount<'info>,
    /// The authority of the user.
    #[account(mut)]
    pub user_authority: UncheckedAccount<'info>,
    /// The input account for LP tokens.
    #[account(mut)]
    pub input_lp: UncheckedAccount<'info>,
    /// The pool mint of the swap.
    #[account(mut)]
    pub pool_mint: UncheckedAccount<'info>,
    /// The token accounts of the user and the token.
    /// The token account associated with the user.
    #[account(mut)]
    pub output_a_user: UncheckedAccount<'info>,
    /// The token account for the pool's reserves of this token.
    #[account(mut)]
    pub output_a_reserve: UncheckedAccount<'info>,
    /// The token account for the fees associated with the token.
    #[account(mut)]
    pub output_a_fees: UncheckedAccount<'info>,
    /// The token accounts of the user and the token.
    /// The token account associated with the user.
    #[account(mut)]
    pub output_b_user: UncheckedAccount<'info>,
    /// The token account for the pool's reserves of this token.
    #[account(mut)]
    pub output_b_reserve: UncheckedAccount<'info>,
    /// The token account for the fees associated with the token.
    #[account(mut)]
    pub output_b_fees: UncheckedAccount<'info>,
    /// The spl_token program.
    pub token_program: Program<'info, Token>,
    /// The clock
    pub clock: Sysvar<'info, Clock>,
}

/// process [saber_withdraw]
impl<'info> SaberWithdraw<'info> {
    /// withdraw from Saber stable swap pool
    pub fn process(&self, args: SaberWithdrawArgs) -> ProgramResult {
        let cpi_accounts = stable_swap_anchor::Withdraw {
            user: stable_swap_anchor::SwapUserContext {
                token_program: self.token_program.to_account_info(),
                swap_authority: self.swap_authority.to_account_info(),
                user_authority: self.user_authority.to_account_info(),
                swap: self.swap.to_account_info(),
                clock: self.clock.to_account_info(),
            },
            input_lp: self.input_lp.to_account_info(),
            pool_mint: self.pool_mint.to_account_info(),
            output_a: stable_swap_anchor::SwapOutput {
                user_token: stable_swap_anchor::SwapToken {
                    user: self.output_a_user.to_account_info(),
                    reserve: self.output_a_reserve.to_account_info(),
                },
                fees: self.output_a_fees.to_account_info(),
            },
            output_b: stable_swap_anchor::SwapOutput {
                user_token: stable_swap_anchor::SwapToken {
                    user: self.output_b_user.to_account_info(),
                    reserve: self.output_b_reserve.to_account_info(),
                },
                fees: self.output_b_fees.to_account_info(),
            },
        };

        let cpi_context = CpiContext::new(self.saber_program.to_account_info(), cpi_accounts);

        self.state.with_vault_auth_seeds(|auth_seeds| {
            stable_swap_anchor::withdraw(
                cpi_context.with_signer(&[auth_seeds]),
                args.pool_token_amount,
                args.minimum_token_a_amount,
                args.minimum_token_b_amount,
            )
        })
    }
}

//-----------------------------------------------------

/// accounts for [saber_withdraw_one]
#[derive(Accounts)]
pub struct SaberWithdrawOne<'info> {
    /// admin
    #[account(mut)]
    pub admin: Signer<'info>,

    /// main state
    #[account(
        has_one = admin @ OneRingFinanceError::AccessDenied,
        constraint = !state.emergency_flag @ OneRingFinanceError::ServiceDisabled,
    )]
    pub state: Box<Account<'info, State>>,

    /// saber stable swap program
    pub saber_program: UncheckedAccount<'info>,

    // cpi accounts below
    /// The swap.
    pub swap: UncheckedAccount<'info>,
    /// The authority of the swap.
    pub swap_authority: UncheckedAccount<'info>,
    /// The authority of the user.
    #[account(mut)]
    pub user_authority: UncheckedAccount<'info>,
    /// The input account for LP tokens.
    #[account(mut)]
    pub input_lp: UncheckedAccount<'info>,
    /// The pool mint of the swap.
    #[account(mut)]
    pub pool_mint: UncheckedAccount<'info>,
    /// Accounts for quote tokens (the token not being withdrawn).
    pub quote_reserves: AccountInfo<'info>,
    /// The token accounts of the user and the token.
    /// The token account associated with the user.
    #[account(mut)]
    pub output_user: UncheckedAccount<'info>,
    /// The token account for the pool's reserves of this token.
    #[account(mut)]
    pub output_reserve: UncheckedAccount<'info>,
    /// The token account for the fees associated with the token.
    #[account(mut)]
    pub output_fees: UncheckedAccount<'info>,
    /// The spl_token program.
    pub token_program: Program<'info, Token>,
    /// The clock
    pub clock: Sysvar<'info, Clock>,
}

/// process [saber_withdraw_one]
impl<'info> SaberWithdrawOne<'info> {
    /// withdraw from Saber stable swap pool
    pub fn process(&self, args: SaberWithdrawOneArgs) -> ProgramResult {
        let cpi_accounts = stable_swap_anchor::WithdrawOne {
            user: stable_swap_anchor::SwapUserContext {
                token_program: self.token_program.to_account_info(),
                swap_authority: self.swap_authority.to_account_info(),
                user_authority: self.user_authority.to_account_info(),
                swap: self.swap.to_account_info(),
                clock: self.clock.to_account_info(),
            },
            input_lp: self.input_lp.to_account_info(),
            pool_mint: self.pool_mint.to_account_info(),
            quote_reserves: self.quote_reserves.to_account_info(),
            output: stable_swap_anchor::SwapOutput {
                user_token: stable_swap_anchor::SwapToken {
                    user: self.output_user.to_account_info(),
                    reserve: self.output_reserve.to_account_info(),
                },
                fees: self.output_fees.to_account_info(),
            },
        };

        let cpi_context = CpiContext::new(self.saber_program.to_account_info(), cpi_accounts);

        self.state.with_vault_auth_seeds(|auth_seeds| {
            stable_swap_anchor::withdraw_one(
                cpi_context.with_signer(&[auth_seeds]),
                args.pool_token_amount,
                args.minimum_token_amount,
            )
        })
    }
}

//-----------------------------------------------------
