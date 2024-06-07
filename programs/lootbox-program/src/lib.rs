use anchor_lang::prelude::*;

declare_id!("E4hHTPaKJdnEocSph1gijTtJon3venE4jdiD4HFKHvaG");

mod error;
mod state;
mod instructions;

use error::*;
use instructions::*;

#[program]
pub mod lootbox_program {
    use super::*;

    pub fn open_lootbox(ctx: Context<OpenLootbox>, box_number: u64) -> Result<()> {
        ctx.accounts.open_lootbox(box_number)?;

        Ok(())
    }

    pub fn retrieve_item_from_lootbox(ctx: Context<RetrieveItem>) -> Result<()> {
        ctx.accounts.retrieve_item_from_lootbox(&ctx.bumps)?;

        Ok(())
    }
}
#[derive(Clone)]
pub struct StakingProgram;

impl anchor_lang::Id for StakingProgram {
    fn id() -> Pubkey {
        "3CUC1Enh3GF7X1vE7ixm1Aq7cv1fTqY7UZvnDoz7X9sZ"
            .parse::<Pubkey>()
            .unwrap()
    }
}
