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
  #[msg("Random mint has not been assigned yet")]
  MintNotReady,
  #[msg("Randomness already requested")]
  RandomnessAlreadyRequested,
  #[msg("Uninitialized Lootbox Pointer")]
  UninitializedLootboxPointer,
}
