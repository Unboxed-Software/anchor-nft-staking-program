use anchor_lang::prelude::*;

#[error_code]
pub enum StakeError {
    #[msg("NFT already staked")]
    AlreadyStaked,

    #[msg("State account is uninitialized")]
    UninitializedAccount,

    #[msg("Stake state is invalid")]
    InvalidStakeState,
}