use anchor_lang::prelude::*;
use anchor_nft_staking::UserStakeInfo;
use anchor_spl::token;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Burn, Mint, MintTo, Token, TokenAccount},
};
pub use switchboard_v2::{
    OracleQueueAccountData, PermissionAccountData, SbState, VrfAccountData, VrfRequestRandomness,
};
pub mod error;
pub mod instructions;
pub mod state;
use error::*;
use instructions::*;
use state::*;

declare_id!("3vnfmoJk6zuEyXt9QfYaCp6psgwe5UL4jLRZKmxThTc9");

#[program]
pub mod lootbox_program {
    use super::*;

    pub fn init_user(ctx: Context<InitUser>, params: InitUserParams) -> Result<()> {
        InitUser::process_instruction(&ctx, &params)
    }

    pub fn open_lootbox(mut ctx: Context<OpenLootbox>, box_number: u64) -> Result<()> {
        OpenLootbox::process_instruction(&mut ctx, box_number)
    }

    pub fn consume_randomness(mut ctx: Context<ConsumeRandomness>) -> Result<()> {
        ConsumeRandomness::process_instruction(&mut ctx)
    }

    pub fn retrieve_item_from_lootbox(mut ctx: Context<RetrieveItem>) -> Result<()> {
        RetrieveItem::process_instruction(&mut ctx)
    }
}
