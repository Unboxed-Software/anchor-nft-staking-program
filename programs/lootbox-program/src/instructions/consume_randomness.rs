use crate::state::*;
use crate::*;

#[derive(Accounts)]
pub struct ConsumeRandomness<'info> {
    #[account(
        mut,
        seeds = [
            payer.key().as_ref(),
        ],
        bump = state.load()?.bump,
        has_one = vrf @ LootboxError::InvalidVrfAccount
    )]
    pub state: AccountLoader<'info, UserState>,
    pub vrf: AccountLoader<'info, VrfAccountData>,
    // #[account(
    //     mut,
    //     seeds=["lootbox".as_bytes(), payer.key().as_ref()],
    //     bump
    //   )]
    // pub lootbox_pointer: Account<'info, LootboxPointer>,
    /// CHECK: not important...
    pub payer: AccountInfo<'info>,
}

impl ConsumeRandomness<'_> {
    pub fn process_instruction(ctx: &mut Context<Self>) -> Result<()> {
        let vrf = ctx.accounts.vrf.load()?;
        let state = &mut ctx.accounts.state.load_mut()?;

        let result_buffer = vrf.get_result()?;
        if result_buffer == [0u8; 32] {
            msg!("vrf buffer empty");
            return Ok(());
        }

        if result_buffer == state.result_buffer {
            msg!("result_buffer unchanged");
            return Ok(());
        }

        let available_gear = vec![
            "EEmw12BYv1nrSGQbQucpV82uDiEWPX26eipDPL8W3rdY"
                .parse::<Pubkey>()
                .unwrap(),
            "BUN755b7SXPy8i8xFgzEfo7mY59VzsCPqg15ojn6bLTy"
                .parse::<Pubkey>()
                .unwrap(),
            "3e2JoTNLwV6vBHzLrsR2H1LSGfSdguotXBabxjvvXuVB"
                .parse::<Pubkey>()
                .unwrap(),
            "GnjZuiLKKkpnytCNUsWyugZixZdhNQ9eyUw3VzUocFJ1"
                .parse::<Pubkey>()
                .unwrap(),
            "AuBz3izVCzPzxJyPNFAKsYZE2gNDkePX7c3zvtMVkr59"
                .parse::<Pubkey>()
                .unwrap(),
        ];

        // maximum value to convert randomness buffer
        let max_result = available_gear.len();
        msg!("Result buffer is {:?}", result_buffer);
        let value: &[u8] = bytemuck::cast_slice(&result_buffer[..]);
        msg!("u128 buffer {:?}", value);
        let result = (value[0] as usize) % max_result;
        msg!("Current VRF Value [1 - {}) = {}!", max_result, result);

        // Add in randomness later for selecting mint
        let mint = available_gear[result];
        msg!("Next mint is {:?}", mint);
        // ctx.accounts.lootbox_pointer.mint = mint;
        // ctx.accounts.lootbox_pointer.mint_is_ready = true;
        // ctx.accounts.lootbox_pointer.claimed = false;
        let mut state = ctx.accounts.state.load_mut()?;
        state.mint = mint;
        state.redeemable = true;

        Ok(())
    }
}
