use crate::*;

#[account]
pub struct LootboxPointer {
  pub mint: Pubkey,
  pub redeemable: bool,
  pub randomness_requested: bool,
  pub available_lootbox: u64,
  pub is_initialized: bool,
}

#[repr(packed)]
#[account(zero_copy)]
#[derive(Default)]
pub struct UserState {
  pub bump: u8,
  pub switchboard_state_bump: u8,
  pub vrf_permission_bump: u8,
  pub result_buffer: [u8; 32],
  pub vrf: Pubkey,
  pub user: Pubkey,
}
