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
    #[account(
    mut,
    seeds=["lootbox".as_bytes(), state.load()?.user.key().as_ref()],
    bump
  )]
    pub lootbox_pointer: Account<'info, LootboxPointer>,
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
            "7J6xbRKnRCkcwfkqj8e2e9ovGxW22oJniUHCygKRvzvr"
                .parse::<Pubkey>()
                .unwrap(),
            "Awd3AYRjbzFhwWc5PQjeGRLUyxvq7RpiLgvLgw1YU8bm"
                .parse::<Pubkey>()
                .unwrap(),
            "Chru2YcHQ5wo9fZz8PVnbvKWmQq3yZcJgdJ8fse5FjiC"
                .parse::<Pubkey>()
                .unwrap(),
            "FFTeyZ277nBa7PE8Vyot7y3S9X5kyC63WDgxHB3EE4fG"
                .parse::<Pubkey>()
                .unwrap(),
            "Bfh4o6CsbF2BKfCT2wCiqCywSEbCGrZ6Y2kJMcFM92s6"
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
        ctx.accounts.lootbox_pointer.mint = mint;
        ctx.accounts.lootbox_pointer.claimed = false;

        Ok(())
    }
}
