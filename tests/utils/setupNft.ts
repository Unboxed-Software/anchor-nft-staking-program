import {
  bundlrStorage,
  keypairIdentity,
  Metaplex,
} from "@metaplex-foundation/js"
import { createMint, getAssociatedTokenAddress } from "@solana/spl-token-real"
import * as anchor from "@project-serum/anchor"

export const setupNft = async (program, payer) => {
  const metaplex = Metaplex.make(program.provider.connection)
    .use(keypairIdentity(payer))
    .use(bundlrStorage())

  const nft = await metaplex
    .nfts()
    .create({
      uri: "",
      name: "Test nft",
      sellerFeeBasisPoints: 0,
    })
    .run()

  console.log("nft metadata pubkey: ", nft.metadataAddress.toBase58())
  console.log("nft token address: ", nft.tokenAddress.toBase58())
  const [delegatedAuthPda] = await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from("authority")],
    program.programId
  )
  const [stakeStatePda] = await anchor.web3.PublicKey.findProgramAddress(
    [payer.publicKey.toBuffer(), nft.tokenAddress.toBuffer()],
    program.programId
  )

  console.log("delegated authority pda: ", delegatedAuthPda.toBase58())
  console.log("stake state pda: ", stakeStatePda.toBase58())
  const [mintAuth] = await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from("mint")],
    program.programId
  )

  const mint = await createMint(
    program.provider.connection,
    payer,
    mintAuth,
    null,
    2
  )
  console.log("Mint pubkey: ", mint.toBase58())

  const tokenAddress = await getAssociatedTokenAddress(mint, payer.publicKey)

  return {
    nft: nft,
    delegatedAuthPda: delegatedAuthPda,
    stakeStatePda: stakeStatePda,
    mint: mint,
    mintAuth: mintAuth,
    tokenAddress: tokenAddress,
  }
}