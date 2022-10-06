import { SwitchboardTestContext } from "@switchboard-xyz/sbv2-utils"
import * as anchor from "@project-serum/anchor"

export const setupSwitchboard = async (provider, program, payer) => {
  // switchboard testing setup
  const switchboard = await SwitchboardTestContext.loadDevnetQueue(
    provider,
    "F8ce7MsckeZAbAGmxjJNetxYXQa9mKr9nnrC3qKubyYy",
    100_000_000
  )

  console.log(switchboard.mint.address.toString())

  await switchboard.oracleHeartbeat()
  const queueData = await switchboard.queue.loadData()
  console.log(`oracleQueue: ${switchboard.queue.publicKey}`)
  console.log(`unpermissionedVrfEnabled: ${queueData.unpermissionedVrfEnabled}`)
  console.log(`# of oracles heartbeating: ${queueData.queue.length}`)
  console.log(
    "\x1b[32m%s\x1b[0m",
    `\u2714 Switchboard localnet environment loaded successfully\n`
  )
  const [lootboxPointerPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("lootbox"), payer.publicKey.toBuffer()],
    program.programId
  )

  return {
    lootboxPointerPda: lootboxPointerPda,
    switchboard: switchboard,
  }
}
