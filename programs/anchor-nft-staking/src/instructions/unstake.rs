use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, 
    metadata::{
        mpl_token_metadata::instructions::{
            ThawDelegatedAccountCpi, 
            ThawDelegatedAccountCpiAccounts
        }, 
        MasterEditionAccount, 
        Metadata
    }, 
    token::{
        mint_to, 
        revoke, 
        Mint, 
        MintTo, 
        Revoke, 
        Token, 
        TokenAccount
    }
};

use crate::{
    StakeError, 
    state::StakeState, 
    state::UserStakeInfo
};

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        token::authority=user
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    pub nft_mint: Account<'info, Mint>,
    pub nft_edition: Account<'info, MasterEditionAccount>,
    #[account(
        mut,
        seeds = [user.key().as_ref(), nft_token_account.key().as_ref()],
        bump,
        constraint = *user.key == stake_state.user_pubkey,
        constraint = nft_token_account.key() == stake_state.token_account
    )]
    pub stake_state: Account<'info, UserStakeInfo>,
    /// CHECK: manual check
    #[account(mut, seeds=["authority".as_bytes().as_ref()], bump)]
    pub program_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub stake_mint: Account<'info, Mint>,
    /// CHECK: manual check
    #[account(seeds = ["mint".as_bytes().as_ref()], bump)]
    pub stake_authority: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer=user,
        associated_token::mint=stake_mint,
        associated_token::authority=user
    )]
    pub user_stake_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub metadata_program: Program<'info, Metadata>,
}

impl<'info> Unstake<'info> {
    pub fn unstake(&mut self, bumps: &UnstakeBumps) -> Result<()> {
        require!(
            self.stake_state.is_initialized,
            StakeError::UninitializedAccount
        );

        require!(
            self.stake_state.stake_state == StakeState::Staked,
            StakeError::InvalidStakeState
        );

        msg!("Thawing token account");
        let seeds = &[
            "authority".as_bytes(),
            &[bumps.program_authority]
        ];
        let signer_seeds = &[&seeds[..]];

        let token_metadata_program = &self.metadata_program.to_account_info();
        let delegate = &self.program_authority.to_account_info();
        let token_account = &self.nft_token_account.to_account_info();
        let edition = &self.nft_edition.to_account_info();
        let mint = &self.nft_mint.to_account_info();
        let token_program = &self.token_program.to_account_info();

        ThawDelegatedAccountCpi::new(token_metadata_program,
            ThawDelegatedAccountCpiAccounts {
                delegate,
                token_account,
                edition,
                mint,
                token_program,
            }
        ).invoke_signed(signer_seeds)?;

        msg!("Revoking delegate");

        let cpi_revoke_program = self.token_program.to_account_info();
        let cpi_revoke_accounts = Revoke {
            source: self.nft_token_account.to_account_info(),
            authority: self.user.to_account_info(),
        };

        let cpi_revoke_ctx = CpiContext::new(cpi_revoke_program, cpi_revoke_accounts);
        revoke(cpi_revoke_ctx)?;

        let clock = Clock::get()?;

        msg!(
            "Stake last redeem: {:?}",
            self.stake_state.last_stake_redeem
        );

        msg!("Current time: {:?}", clock.unix_timestamp);
        let unix_time = clock.unix_timestamp - self.stake_state.last_stake_redeem;
        msg!("Seconds since last redeem: {}", unix_time);
        let redeem_amount = (10 * i64::pow(10, 2) * unix_time) / (24 * 60 * 60);
        msg!("Elligible redeem amount: {}", redeem_amount);

        msg!("Minting staking rewards");
        mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                MintTo {
                    mint: self.stake_mint.to_account_info(),
                    to: self.user_stake_ata.to_account_info(),
                    authority: self.stake_authority.to_account_info(),
                },
                &[&[
                    b"mint".as_ref(),
                    &[bumps.stake_authority],
                ]],
            ),
            redeem_amount.try_into().unwrap(),
        )?;

        self.stake_state.last_stake_redeem = clock.unix_timestamp;
        msg!(
            "Updated last stake redeem time: {:?}",
            self.stake_state.last_stake_redeem
        );

        self.stake_state.stake_state = StakeState::Unstaked;

        Ok(())
    }
}