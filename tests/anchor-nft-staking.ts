import { AnchorNftStaking } from "../target/types/anchor_nft_staking"
import { setupNft } from "./utils/setupNft"
import { PROGRAM_ID as METADATA_PROGRAM_ID } from "@metaplex-foundation/mpl-token-metadata"
import * as anchor from "@project-serum/anchor"
import { Program } from "@project-serum/anchor"
import { LootboxProgram } from "../target/types/lootbox_program"
import {
  getOrCreateAssociatedTokenAccount,
  getAssociatedTokenAddress,
  getAccount,
  createMint,
  mintToChecked,
} from "@solana/spl-token-real"
import { expect } from "chai"
import { BN } from "@project-serum/anchor"
import { SwitchboardTestContext } from "@switchboard-xyz/sbv2-utils"
import * as sbv2 from "@switchboard-xyz/switchboard-v2"

describe("anchor-nft-staking", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)

  const program = anchor.workspace.AnchorNftStaking as Program<AnchorNftStaking>

  const wallet = anchor.workspace.AnchorNftStaking.provider.wallet

  const lootboxProgram = anchor.workspace
    .LootboxProgram as Program<LootboxProgram>

  let delegatedAuthPda: anchor.web3.PublicKey
  let stakeStatePda: anchor.web3.PublicKey
  let nft: any
  let mintAuth: anchor.web3.PublicKey
  let mint: anchor.web3.PublicKey
  let tokenAddress: anchor.web3.PublicKey

  let switchboard: SwitchboardTestContext
  let userState: anchor.web3.PublicKey
  let userStateBump: number
  let lootboxPointerPda: anchor.web3.PublicKey

  before(async () => {
    ;({ nft, delegatedAuthPda, stakeStatePda, mint, mintAuth, tokenAddress } =
      await setupNft(program, wallet.payer))

    // switchboard testing setup
    switchboard = await SwitchboardTestContext.loadDevnetQueue(
      provider,
      "F8ce7MsckeZAbAGmxjJNetxYXQa9mKr9nnrC3qKubyYy",
      100_000_000
    )

    console.log(switchboard.mint.address.toString())
    // switchboard = await SwitchboardTestContext.loadFromEnv(
    //   program.provider as anchor.AnchorProvider,
    //   undefined,
    //   5_000_000 // .005 wSOL
    // )
    await switchboard.oracleHeartbeat()
    const queueData = await switchboard.queue.loadData()
    console.log(`oracleQueue: ${switchboard.queue.publicKey}`)
    console.log(
      `unpermissionedVrfEnabled: ${queueData.unpermissionedVrfEnabled}`
    )
    console.log(`# of oracles heartbeating: ${queueData.queue.length}`)
    console.log(
      "\x1b[32m%s\x1b[0m",
      `\u2714 Switchboard localnet environment loaded successfully\n`
    )
    ;[lootboxPointerPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("lootbox"), wallet.publicKey.toBuffer()],
      lootboxProgram.programId
    )
  })

  it("Stakes", async () => {
    // Add your test here.
    await program.methods
      .stake()
      .accounts({
        nftTokenAccount: nft.tokenAddress,
        nftMint: nft.mintAddress,
        nftEdition: nft.masterEditionAddress,
        metadataProgram: METADATA_PROGRAM_ID,
      })
      .rpc()

    const account = await program.account.userStakeInfo.fetch(stakeStatePda)
    expect(account.stakeState.staked)
    expect(Number(account.stakeState.totalEarned) === 0)
  })

  it("Redeems", async () => {
    await program.methods
      .redeem()
      .accounts({
        nftTokenAccount: nft.tokenAddress,
        stakeMint: mint,
        userStakeAta: tokenAddress,
      })
      .rpc()

    const account = await program.account.userStakeInfo.fetch(stakeStatePda)
    expect(account.stakeState.staked).to.not.equal(undefined)
    const tokenAccount = await getAccount(provider.connection, tokenAddress)
    expect(Number(tokenAccount.amount) > 0)
  })

  it("Unstakes", async () => {
    await program.methods
      .unstake()
      .accounts({
        nftTokenAccount: nft.tokenAddress,
        nftMint: nft.mintAddress,
        nftEdition: nft.masterEditionAddress,
        metadataProgram: METADATA_PROGRAM_ID,
        stakeMint: mint,
        userStakeAta: tokenAddress,
      })
      .rpc()

    const account = await program.account.userStakeInfo.fetch(stakeStatePda)
    expect(account.stakeState.unstaked).to.not.equal(undefined)
  })

  it("init user", async () => {
    const { unpermissionedVrfEnabled, authority, dataBuffer } =
      await switchboard.queue.loadData()

    // keypair for vrf account
    const vrfKeypair = anchor.web3.Keypair.generate()

    // find PDA used for our client state pubkey
    ;[userState, userStateBump] = anchor.utils.publicKey.findProgramAddressSync(
      [wallet.publicKey.toBytes()],
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
          { pubkey: lootboxPointerPda, isSigner: false, isWritable: false },
          { pubkey: wallet.publicKey, isSigner: false, isWritable: false },
        ],
        ixData: new anchor.BorshInstructionCoder(lootboxProgram.idl).encode(
          "consumeRandomness",
          ""
        ),
      },
    })
    console.log(`Created VRF Account: ${vrfAccount.publicKey}`)

    // create permissionAccount
    const permissionAccount = await sbv2.PermissionAccount.create(
      switchboard.program,
      {
        authority,
        granter: switchboard.queue.publicKey,
        grantee: vrfAccount.publicKey,
      }
    )
    console.log(`Created Permission Account: ${permissionAccount.publicKey}`)

    // If queue requires permissions to use VRF, check the correct authority was provided
    if (!unpermissionedVrfEnabled) {
      if (!wallet.publicKey.equals(authority)) {
        throw new Error(
          `queue requires PERMIT_VRF_REQUESTS and wrong queue authority provided`
        )
      }

      await permissionAccount.set({
        authority: wallet.payer,
        permission: sbv2.SwitchboardPermission.PERMIT_VRF_REQUESTS,
        enable: true,
      })
      console.log(`Set VRF Permissions`)
    }

    const vrfState = await vrfAccount.loadData()
    const queueAccount = new sbv2.OracleQueueAccount({
      program: switchboard.program,
      publicKey: vrfState.oracleQueue,
    })

    const queueState = await queueAccount.loadData()

    const [_permissionAccount, permissionBump] =
      sbv2.PermissionAccount.fromSeed(
        switchboard.program,
        queueState.authority,
        queueAccount.publicKey,
        vrfAccount.publicKey
      )

    const [_programStateAccount, switchboardStateBump] =
      sbv2.ProgramStateAccount.fromSeed(switchboard.program)

    const tx = await lootboxProgram.methods
      .initUser({
        switchboardStateBump: switchboardStateBump,
        vrfPermissionBump: permissionBump,
      })
      .accounts({
        state: userState,
        vrf: vrfAccount.publicKey,
        payer: wallet.pubkey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc()

    console.log(`https://explorer.solana.com/tx/${tx}?cluster=devnet`)
  })

  it("Chooses a mint pseudorandomly", async () => {
    const mint = await createMint(
      provider.connection,
      wallet.payer,
      wallet.publicKey,
      wallet.publicKey,
      2
    )
    const ata = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      wallet.payer,
      mint,
      wallet.publicKey
    )

    await mintToChecked(
      provider.connection,
      wallet.payer,
      mint,
      ata.address,
      wallet.payer,
      1000,
      2
    )

    const [stakeAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [wallet.publicKey.toBuffer(), nft.tokenAddress.toBuffer()],
      program.programId
    )

    await lootboxProgram.methods
      .openLootbox(new BN(10))
      .accounts({
        stakeMint: mint,
        stakeMintAta: ata.address,
        stakeState: stakeAccount,
      })
      .rpc()

    const [address] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("lootbox"), wallet.publicKey.toBuffer()],
      lootboxProgram.programId
    )
    const pointer = await lootboxProgram.account.lootboxPointer.fetch(address)
    expect(pointer.mint.toBase58())
  })

  it("Mints the selected gear", async () => {
    const [pointerAddress] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("lootbox"), wallet.publicKey.toBuffer()],
      lootboxProgram.programId
    )

    const pointer = await lootboxProgram.account.lootboxPointer.fetch(
      pointerAddress
    )

    const gearAta = await getAssociatedTokenAddress(
      pointer.mint,
      wallet.publicKey
    )
    await lootboxProgram.methods
      .retrieveItemFromLootbox()
      .accounts({
        mint: pointer.mint,
        userGearAta: gearAta,
      })
      .rpc()

    const gearAccount = await getAccount(provider.connection, gearAta)
    expect(Number(gearAccount.amount)).to.equal(1)
  })
})
