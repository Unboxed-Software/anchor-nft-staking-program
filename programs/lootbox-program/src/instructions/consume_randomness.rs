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
