use crate::*;

#[error_code]
pub enum LootboxError {
  #[msg("Mint already claimed")]
  AlreadyClaimed,
  #[msg("Haven't staked long enough for this loot box or invalid loot box number")]
  InvalidLootbox,
  #[msg("Switchboard VRF Account's authority should be set to the client's state pubkey")]
  InvalidVrfAuthorityError,
  #[msg("Invalid VRF account provided.")]
  InvalidVrfAccount,
}
