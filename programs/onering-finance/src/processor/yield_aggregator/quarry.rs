use anchor_lang::prelude::*;
use anchor_spl::token::Token;

use crate::{args::*, error::*, states::*, traits::*};

#[derive(Accounts, Clone)]
pub struct QuarryUserStake<'info> {
    /// admin
    pub admin: Signer<'info>,

    /// main state
    #[account(
        has_one = admin @ CommonError::AccessDenied,
        constraint = !state.emergency_flag @ CommonError::ServiceDisabled,
    )]
    pub state: Box<Account<'info, State>>,

    /// quarry mine program
    pub quarry_mine_program: UncheckedAccount<'info>,

    // cpi accounts below
    /// Miner authority (i.e. the user).
    pub authority: UncheckedAccount<'info>,
    /// Miner.
    #[account(mut)]
    pub miner: UncheckedAccount<'info>,
    /// Quarry to claim from.
    #[account(mut)]
    pub quarry: UncheckedAccount<'info>,
    /// Vault of the miner.
    #[account(mut)]
    pub miner_vault: UncheckedAccount<'info>,
    /// User's staked token account
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,
    /// Token program
    pub token_program: Program<'info, Token>,
    /// Rewarder
    pub rewarder: UncheckedAccount<'info>,
}

impl<'info> QuarryUserStake<'info> {
    /// UserStake CpiContext
    fn to_user_stake_cpi_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, quarry_mine::cpi::accounts::UserStake<'info>> {
        let cpi_accounts = quarry_mine::cpi::accounts::UserStake {
            /// Miner authority (i.e. the user).
            authority: self.authority.to_account_info(),
            /// Miner.
            miner: self.miner.to_account_info(),
            /// Quarry to claim from.
            quarry: self.quarry.to_account_info(),
            /// Vault of the miner.
            miner_vault: self.miner_vault.to_account_info(),
            /// User's staked token account
            token_account: self.token_account.to_account_info(),
            /// Token program
            token_program: self.token_program.to_account_info(),
            /// Rewarder
            rewarder: self.rewarder.to_account_info(),
        };

        CpiContext::new(self.quarry_mine_program.to_account_info(), cpi_accounts)
    }

    /// process [stake_tokens]
    pub fn process_stake_tokens(&self, args: DepositOrWithdrawArgs) -> ProgramResult {
        self.state.with_vault_auth_seeds(|auth_seeds| {
            quarry_mine::cpi::stake_tokens(
                self.to_user_stake_cpi_context().with_signer(&[auth_seeds]),
                args.amount,
            )
        })
    }

    /// process [withdraw_tokens]
    pub fn process_withdraw_tokens(&self, args: DepositOrWithdrawArgs) -> ProgramResult {
        self.state.with_vault_auth_seeds(|auth_seeds| {
            quarry_mine::cpi::withdraw_tokens(
                self.to_user_stake_cpi_context().with_signer(&[auth_seeds]),
                args.amount,
            )
        })
    }
}
