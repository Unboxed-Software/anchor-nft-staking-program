use anchor_lang::prelude::*;

mod state;
mod instructions;
mod error;

use state::*;
use instructions::*;
use error::*;

declare_id!("7X136wosjGtZXSQjnsf7qXbENUWAYrRGPppYdPVv3Z4E");

#[program]
pub mod anchor_nft_staking {

    use super::*;

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        ctx.accounts.stake(&ctx.bumps)?;

        Ok(())
    }

    pub fn redeem(ctx: Context<Redeem>) -> Result<()> {
        ctx.accounts.redeem(&ctx.bumps)?;

        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        ctx.accounts.unstake(&ctx.bumps)?;

        Ok(())
    }
}
