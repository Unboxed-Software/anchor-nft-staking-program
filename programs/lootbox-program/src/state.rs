use crate::*;

// #[account]
// pub struct LootboxPointer {
//   pub mint: Pubkey,
//   pub mint_is_ready: bool,
//   pub claimed: bool,
//   pub is_initialized: bool,
// }

#[repr(packed)]
#[account(zero_copy)]
#[derive(Default)]
pub struct UserState {
  pub bump: u8,
  pub switchboard_state_bump: u8,
  pub vrf_permission_bump: u8,
  pub result_buffer: [u8; 32],
  pub result: u128,
  pub vrf: Pubkey,
  pub mint: Pubkey,
  pub token_account: Pubkey,
  pub redeemable: bool,
  pub user: Pubkey,
}
