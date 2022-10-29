use crate::*;
use anchor_lang::solana_program;

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
    pub lootbox_pointer: Box<Account<'info, LootboxPointer>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    // TESTING - Uncomment the next line during testing
    // #[account(mut)]
    // TESTING - Comment out the next three lines during testing
    #[account(
          mut,
          address="D7F9JnGcjxQwz9zEQmasksX1VrwFcfRKu8Vdqrk2enHR".parse::<Pubkey>().unwrap()
      )]
    pub stake_mint: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint=stake_mint,
        associated_token::authority=user
    )]
    pub stake_mint_ata: Box<Account<'info, TokenAccount>>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(
        constraint=stake_state.user_pubkey==user.key(),
    )]
    pub stake_state: Box<Account<'info, UserStakeInfo>>,
    #[account(
        mut,
        // TESTING - Comment out these seeds for testing
        seeds = [
            user.key().as_ref(),
        ],
        // TESTING - Uncomment these seeds for testing
        // seeds = [
        //     vrf.key().as_ref(),
        //     user.key().as_ref()
        // ],
        bump = state.load()?.bump,
        has_one = vrf @ LootboxError::InvalidVrfAccount
    )]
    pub state: AccountLoader<'info, UserState>,

    // SWITCHBOARD ACCOUNTS
    #[account(mut,
        has_one = escrow
    )]
    pub vrf: AccountLoader<'info, VrfAccountData>,
    #[account(mut,
        has_one = data_buffer
    )]
    pub oracle_queue: AccountLoader<'info, OracleQueueAccountData>,
    /// CHECK:
    #[account(mut,
        constraint =
            oracle_queue.load()?.authority == queue_authority.key()
    )]
    pub queue_authority: UncheckedAccount<'info>,
    /// CHECK
    #[account(mut)]
    pub data_buffer: AccountInfo<'info>,
    #[account(mut)]
    pub permission: AccountLoader<'info, PermissionAccountData>,
    #[account(mut,
        constraint =
            escrow.owner == program_state.key()
            && escrow.mint == program_state.load()?.token_mint
    )]
    pub escrow: Account<'info, TokenAccount>,
    #[account(mut)]
    pub program_state: AccountLoader<'info, SbState>,
    /// CHECK:
    #[account(
        address = *vrf.to_account_info().owner,
        constraint = switchboard_program.executable == true
    )]
    pub switchboard_program: AccountInfo<'info>,

    // PAYER ACCOUNTS
    #[account(mut,
        constraint =
            payer_wallet.owner == user.key()
            && escrow.mint == program_state.load()?.token_mint
    )]
    pub payer_wallet: Account<'info, TokenAccount>,
    // SYSTEM ACCOUNTS
    /// CHECK:
    #[account(address = solana_program::sysvar::recent_blockhashes::ID)]
    pub recent_blockhashes: AccountInfo<'info>,
}

#[derive(Clone)]
pub struct StakingProgram;

impl anchor_lang::Id for StakingProgram {
    fn id() -> Pubkey {
        "2uE2DSDFoz9qendAdDpFL4wQ79cX2M4m3DFGX41KQ5YX"
            .parse::<Pubkey>()
            .unwrap()
    }
}

impl OpenLootbox<'_> {
    pub fn process_instruction(ctx: &mut Context<Self>, box_number: u64) -> Result<()> {
        if ctx.accounts.lootbox_pointer.available_lootbox == 0 {
            ctx.accounts.lootbox_pointer.available_lootbox = 10;
        }
        require!(
            ctx.accounts.stake_state.total_earned >= box_number
                && ctx.accounts.lootbox_pointer.available_lootbox == box_number,
            LootboxError::InvalidLootbox
        );

        require!(
            !ctx.accounts.lootbox_pointer.randomness_requested,
            LootboxError::RandomnessAlreadyRequested
        );

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

        let state = ctx.accounts.state.load()?;
        let bump = state.bump.clone();
        let switchboard_state_bump = state.switchboard_state_bump;
        let vrf_permission_bump = state.vrf_permission_bump;
        drop(state);

        let switchboard_program = ctx.accounts.switchboard_program.to_account_info();

        let vrf_request_randomness = VrfRequestRandomness {
            authority: ctx.accounts.state.to_account_info(),
            vrf: ctx.accounts.vrf.to_account_info(),
            oracle_queue: ctx.accounts.oracle_queue.to_account_info(),
            queue_authority: ctx.accounts.queue_authority.to_account_info(),
            data_buffer: ctx.accounts.data_buffer.to_account_info(),
            permission: ctx.accounts.permission.to_account_info(),
            escrow: ctx.accounts.escrow.clone(),
            payer_wallet: ctx.accounts.payer_wallet.clone(),
            payer_authority: ctx.accounts.user.to_account_info(),
            recent_blockhashes: ctx.accounts.recent_blockhashes.to_account_info(),
            program_state: ctx.accounts.program_state.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };

        let payer = ctx.accounts.user.key();
        // TESTING - uncomment the following during tests
        // let vrf = ctx.accounts.vrf.key();
        // let state_seeds: &[&[&[u8]]] = &[&[vrf.as_ref(), payer.as_ref(), &[bump]]];
        // TESTING - comment out the next line during tests
        let state_seeds: &[&[&[u8]]] = &[&[payer.as_ref(), &[bump]]];

        msg!("requesting randomness");
        vrf_request_randomness.invoke_signed(
            switchboard_program,
            switchboard_state_bump,
            vrf_permission_bump,
            state_seeds,
        )?;

        msg!("randomness requested successfully");

        ctx.accounts.lootbox_pointer.randomness_requested = true;
        ctx.accounts.lootbox_pointer.is_initialized = true;
        ctx.accounts.lootbox_pointer.available_lootbox = box_number * 2;

        Ok(())
    }
}
