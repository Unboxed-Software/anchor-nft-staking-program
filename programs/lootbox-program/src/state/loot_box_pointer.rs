use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct LootboxPointer {
    pub mint: Pubkey,
    pub claimed: bool,
    pub is_initialized: bool,
}