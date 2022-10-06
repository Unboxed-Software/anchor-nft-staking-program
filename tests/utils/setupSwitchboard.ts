import { SwitchboardTestContext } from "@switchboard-xyz/sbv2-utils"
import * as anchor from "@project-serum/anchor"
import * as sbv2 from "@switchboard-xyz/switchboard-v2"

export const setupSwitchboard = async (provider, lootboxProgram, payer) => {
  // switchboard testing setup

  const switchboard = await SwitchboardTestContext.loadDevnetQueue(
    provider,
    "F8ce7MsckeZAbAGmxjJNetxYXQa9mKr9nnrC3qKubyYy",
    100_000_000
  )

  console.log(switchboard.mint.address.toString())

  await switchboard.oracleHeartbeat()
  const { queue, unpermissionedVrfEnabled, authority } =
    await switchboard.queue.loadData()
  console.log(`oracleQueue: ${switchboard.queue.publicKey}`)
  console.log(`unpermissionedVrfEnabled: ${unpermissionedVrfEnabled}`)
  console.log(`# of oracles heartbeating: ${queue.length}`)
  console.log(
    "\x1b[32m%s\x1b[0m",
    `\u2714 Switchboard devnet environment loaded successfully\n`
  )

  // CREATE VRF ACCOUNT
  // keypair for vrf account
  const vrfKeypair = anchor.web3.Keypair.generate()

  // find PDA used for our client state pubkey
  const [userState] = anchor.utils.publicKey.findProgramAddressSync(
    [vrfKeypair.publicKey.toBytes(), payer.publicKey.toBytes()],
    lootboxProgram.programId
  )

  // lootboxPointerPda for callback
  const [lootboxPointerPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("lootbox"), payer.publicKey.toBuffer()],
    lootboxProgram.programId
  )

  // create new vrf acount
  const vrfAccount = await sbv2.VrfAccount.create(switchboard.program, {
    keypair: vrfKeypair,
    authority: userState, // set vrfAccount authority as PDA
    queue: switchboard.queue,
    callback: {
      programId: lootboxProgram.programId,
      accounts: [
        { pubkey: userState, isSigner: false, isWritable: true },
        { pubkey: vrfKeypair.publicKey, isSigner: false, isWritable: false },
        { pubkey: lootboxPointerPda, isSigner: false, isWritable: true },
        { pubkey: payer.publicKey, isSigner: false, isWritable: false },
      ],
      ixData: new anchor.BorshInstructionCoder(lootboxProgram.idl).encode(
        "consumeRandomness",
        ""
      ),
    },
  })

  // CREATE PERMISSION ACCOUNT
  const permissionAccount = await sbv2.PermissionAccount.create(
    switchboard.program,
    {
      authority,
      granter: switchboard.queue.publicKey,
      grantee: vrfAccount.publicKey,
    }
  )

  // If queue requires permissions to use VRF, check the correct authority was provided
  if (!unpermissionedVrfEnabled) {
    if (!payer.publicKey.equals(authority)) {
      throw new Error(
        `queue requires PERMIT_VRF_REQUESTS and wrong queue authority provided`
      )
    }

    await permissionAccount.set({
      authority: payer,
      permission: sbv2.SwitchboardPermission.PERMIT_VRF_REQUESTS,
      enable: true,
    })
  }

  // GET PERMISSION BUMP AND SWITCHBOARD STATE BUMP
  const [_permissionAccount, permissionBump] = sbv2.PermissionAccount.fromSeed(
    switchboard.program,
    authority,
    switchboard.queue.publicKey,
    vrfAccount.publicKey
  )

  const [switchboardStateAccount, switchboardStateBump] =
    sbv2.ProgramStateAccount.fromSeed(switchboard.program)

  return {
    switchboard: switchboard,
    lootboxPointerPda: lootboxPointerPda,
    permissionBump: permissionBump,
    permissionAccount: permissionAccount,
    switchboardStateBump: switchboardStateBump,
    switchboardStateAccount: switchboardStateAccount,
    vrfAccount: vrfAccount,
  }
}
