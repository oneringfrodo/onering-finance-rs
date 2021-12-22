use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

use crate::{error::*, states::*, traits::*};

//-----------------------------------------------------

/// accounts for claim rewards
#[derive(Accounts)]
#[instruction(args: ClaimArgs)]
pub struct Claim<'info> {
    /// user, reserve initializer
    #[account()]
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
}

/// claim for rewards
impl<'info> Processor<ClaimArgs> for Claim<'info> {
    fn process(&mut self, args: ClaimArgs) -> ProgramResult {
        // refresh reserve state
        self.reserve.refresh_reserve(&mut self.state);

        // check if claim amount less than reward amount
        if self.reserve.reward_amount < args.amount {
            return Err(CommonError::ClaimAmountTooMuch.into());
        }

        // reduct reward amount
        self.reserve.reward_amount -= args.amount;

        // mint claim amount of 1USD to initializer
        self.mint_to_initializer(args.amount)?;

        Ok(())
    }
}

//-----------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct ClaimArgs {
    pub amount: u64,
}
