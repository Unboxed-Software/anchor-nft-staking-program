use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct UserStakeInfo {
    pub token_account: Pubkey,
    pub stake_start_time: i64,
    pub last_stake_redeem: i64,
    pub user_pubkey: Pubkey,
    pub stake_state: StakeState,
    pub is_initialized: bool,
}

#[derive(Debug, PartialEq, AnchorDeserialize, AnchorSerialize, Clone, InitSpace)]
pub enum StakeState {
    Unstaked,
    Staked,
}

impl Default for StakeState {
    fn default() -> Self {
        StakeState::Unstaked
    }
}