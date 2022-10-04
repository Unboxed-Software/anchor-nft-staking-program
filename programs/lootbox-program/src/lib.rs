use anchor_lang::prelude::*;
use anchor_nft_staking::UserStakeInfo;
use anchor_spl::token;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Burn, Mint, MintTo, Token, TokenAccount},
};

declare_id!("6GE3ki2igpw2ZTAt6BV4pTjF5qvtCbFVQP7SGPJaEuoa");

#[program]
pub mod lootbox_program {
    use super::*;

    pub fn open_lootbox(ctx: Context<OpenLootbox>, box_number: u64) -> Result<()> {
        let mut loot_box = 10;
        loop {
            if loot_box > box_number {
                return err!(LootboxError::InvalidLootbox);
            }

            if loot_box == box_number {
                require!(
                    ctx.accounts.stake_state.total_earned >= box_number,
                    LootboxError::InvalidLootbox
                );
                break;
            } else {
                loot_box = loot_box * 2;
            }
        }

        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.stake_mint.to_account_info(),
                    from: ctx.accounts.stake_mint_ata.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
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
        ctx.accounts.lootbox_pointer.mint = mint;
        ctx.accounts.lootbox_pointer.claimed = false;
        ctx.accounts.lootbox_pointer.is_initialized = true;

        Ok(())
    }

    pub fn retrieve_item_from_lootbox(ctx: Context<RetrieveItem>) -> Result<()> {
        require!(
            !ctx.accounts.lootbox_pointer.claimed,
            LootboxError::AlreadyClaimed
        );
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.user_gear_ata.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                &[&[
                    b"mint".as_ref(),
                    &[*ctx.bumps.get("mint_authority").unwrap()],
                ]],
            ),
            1,
        )?;

        ctx.accounts.lootbox_pointer.claimed = true;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct OpenLootbox<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init_if_needed,
        payer = user,
        space = std::mem::size_of::<LootboxPointer>() + 8,
        seeds=["lootbox".as_bytes(), user.key().as_ref()],
        bump
    )]
    pub lootbox_pointer: Account<'info, LootboxPointer>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    // Swap the next two lines out between prod/testing
    // #[account(mut)]
    #[account(
        mut,
        address="6YR1nuLqkk8VC1v42xJaPKvE9X9pnuqVAvthFUSDsMUL".parse::<Pubkey>().unwrap()
    )]
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

#[derive(Clone)]
pub struct StakingProgram;

impl anchor_lang::Id for StakingProgram {
    fn id() -> Pubkey {
        "3CUC1Enh3GF7X1vE7ixm1Aq7cv1fTqY7UZvnDoz7X9sZ"
            .parse::<Pubkey>()
            .unwrap()
    }
}

#[derive(Accounts)]
pub struct RetrieveItem<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        seeds=["lootbox".as_bytes(), user.key().as_ref()],
        bump,
        constraint=lootbox_pointer.is_initialized
    )]
    pub lootbox_pointer: Account<'info, LootboxPointer>,
    #[account(
        mut,
        constraint=lootbox_pointer.mint==mint.key()
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer=user,
        associated_token::mint=mint,
        associated_token::authority=user
    )]
    pub user_gear_ata: Account<'info, TokenAccount>,
    /// CHECK: Mint authority - not used as account
    #[account(
        seeds=["mint".as_bytes()],
        bump
    )]
    pub mint_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
pub struct LootboxPointer {
    mint: Pubkey,
    claimed: bool,
    is_initialized: bool,
}

#[error_code]
enum LootboxError {
    #[msg("Mint already claimed")]
    AlreadyClaimed,

    #[msg("Haven't staked long enough for this loot box or invalid loot box number")]
    InvalidLootbox,
}
