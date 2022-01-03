use anchor_lang::prelude::*;

//-----------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct CreateReserveArgs {
    pub reserve_bump: Option<u8>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct DepositOrWithdrawArgs {
    pub amount: u64,
}

//-----------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct CreateMarketArgs {
    pub stable_vault_bump: u8,
}

//-----------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct CreateAdminArgs {
    pub ousd_mint_auth_bump: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct UpdateAdminArgs {}

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct UpdateStateArgs {
    pub emergency_flag: bool,
}

//-----------------------------------------------------
