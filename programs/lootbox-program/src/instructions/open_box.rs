use anchor_lang::prelude::*;
use anchor_nft_staking::state::UserStakeInfo;
use anchor_spl::{
    associated_token::AssociatedToken, 
    token::{
        burn, 
        Burn, 
        Mint, 
        Token, 
        TokenAccount
    }
};

use crate::{
    state::LootboxPointer, 
    LootboxError
};

#[derive(Accounts)]
pub struct OpenLootbox<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + LootboxPointer::INIT_SPACE,
        seeds=["lootbox".as_bytes(), user.key().as_ref()],
        bump
    )]
    pub lootbox_pointer: Account<'info, LootboxPointer>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    #[account(mut)]
    pub stake_mint: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint=stake_mint,
        associated_token::authority=user
    )]
    pub stake_mint_ata: Account<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(
        constraint=stake_state.user_pubkey==user.key(),
    )]
    pub stake_state: Account<'info, UserStakeInfo>,
}

impl<'info> OpenLootbox<'info> {
    pub fn open_lootbox(&mut self, box_number: u64) -> Result<()> {
        let mut loot_box = 10;
        loop {
            if loot_box > box_number {
                return err!(LootboxError::InvalidLootbox);
            }

            if loot_box == box_number {
                require!(
                    self.stake_state.total_earned >= box_number,
                    LootboxError::InvalidLootbox
                );
                break;
            } else {
                loot_box = loot_box * 2;
            }
        }

        require!(
            !self.lootbox_pointer.is_initialized || self.lootbox_pointer.claimed,
            LootboxError::InvalidLootbox
        );

        burn(
            CpiContext::new(
                self.token_program.to_account_info(),
                Burn {
                    mint: self.stake_mint.to_account_info(),
                    from: self.stake_mint_ata.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            ),
            box_number * u64::pow(10, 2),
        )?;

        let available_gear: Vec<Pubkey> = vec![
            "DQmrQJkErmfe6a1fD2hPwdLSnawzkdyrKfSUmd6vkC89"
                .parse::<Pubkey>()
                .unwrap(),
            "A26dg2NBfGgU6gpFPfsiLpxwsV13ZKiD58zgjeQvuad"
                .parse::<Pubkey>()
                .unwrap(),
            "GxR5UVvQDRwB19bCsB1wJh6RtLRZUbEAigtgeAsm6J7N"
                .parse::<Pubkey>()
                .unwrap(),
            "3rL2p6LsGyHVn3iwQQYV9bBmchxMHYPice6ntp7Qw8Pa"
                .parse::<Pubkey>()
                .unwrap(),
            "73JnegAtAWHmBYL7pipcSTpQkkAx77pqCQaEys2Qmrb2"
                .parse::<Pubkey>()
                .unwrap(),
        ];

        let clock = Clock::get()?;
        let i: usize = (clock.unix_timestamp % 5).try_into().unwrap();
        // Add in randomness later for selecting mint
        let mint = available_gear[i];
        self.lootbox_pointer.mint = mint;
        self.lootbox_pointer.claimed = false;
        self.lootbox_pointer.is_initialized = true;

        Ok(())
    }
}