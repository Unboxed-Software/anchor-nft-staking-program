use anchor_lang::prelude::*;

declare_id!("FdCeWWMY2VzNkj5wg7Qiwq8VEDSYjfTaLtYR5jiWfnST");

mod state;
mod instructions;
mod error;

use instructions::*;
use error::*;

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