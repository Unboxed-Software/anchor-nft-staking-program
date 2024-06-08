use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::{
        mpl_token_metadata::instructions::{
            FreezeDelegatedAccountCpi, 
            FreezeDelegatedAccountCpiAccounts
        }, 
        MasterEditionAccount, 
        Metadata
    }, 
    token::{
        approve, 
        Approve, 
        Mint, 
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
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        associated_token::mint=nft_mint,
        associated_token::authority=user
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    pub nft_mint: Account<'info, Mint>,
    pub nft_edition: Account<'info, MasterEditionAccount>,
    #[account(
        init_if_needed,
        payer=user,
        space = 8 + UserStakeInfo::INIT_SPACE,
        seeds = [user.key().as_ref(), nft_token_account.key().as_ref()],
        bump
    )]
    pub stake_state: Account<'info, UserStakeInfo>,
    /// CHECK: Manual validation
    #[account(mut, seeds=["authority".as_bytes().as_ref()], bump)]
    pub program_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub metadata_program: Program<'info, Metadata>,
}

impl<'info> Stake<'info> {
    pub fn stake(&mut self, bumps: &StakeBumps) -> Result<()> {
        require!(
            self.stake_state.stake_state == StakeState::Unstaked,
            StakeError::AlreadyStaked
        );

        let clock = Clock::get().unwrap();
        msg!("Approving delegate");

        let cpi_approve_program = self.token_program.to_account_info();
        let cpi_approve_accounts = Approve {
            to: self.nft_token_account.to_account_info(),
            delegate: self.program_authority.to_account_info(),
            authority: self.user.to_account_info(),
        };

        let cpi_approve_ctx = CpiContext::new(cpi_approve_program, cpi_approve_accounts);
        approve(cpi_approve_ctx, 1)?;

        msg!("Freezing token account");

        // Seeds for the CPI
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

        FreezeDelegatedAccountCpi::new(token_metadata_program,
            FreezeDelegatedAccountCpiAccounts {
                delegate,
                token_account,
                edition,
                mint,
                token_program,
            }
        ).invoke_signed(signer_seeds)?;

        self.stake_state.token_account = self.nft_token_account.key();
        self.stake_state.user_pubkey = self.user.key();
        self.stake_state.stake_state = StakeState::Staked;
        self.stake_state.stake_start_time = clock.unix_timestamp;
        self.stake_state.last_stake_redeem = clock.unix_timestamp;
        self.stake_state.is_initialized = true;

        Ok(())
    }
}