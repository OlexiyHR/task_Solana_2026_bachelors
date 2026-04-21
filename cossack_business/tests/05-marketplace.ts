import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey, SYSVAR_INSTRUCTIONS_PUBKEY, SystemProgram } from "@solana/web3.js";
import { getAssociatedTokenAddressSync, createAssociatedTokenAccountInstruction } from "@solana/spl-token";
import { expect } from "chai";
import { anchorProvider, resourceMgrProg, currencyProg, artifactProg, forgeProg, tradeProg, player1, player2,
  materialKeypairs, tradeCpiPda, artifactValues, bootstrapTestEnv } from "./helpers/setup";
import { TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, METAPLEX_PROGRAM_ID,
  findMetadataPda, findMasterEditionPda, doMultipleForaging, forgeArtifactForPlayer } from "./helpers/utils";

describe("Marketplace & Trade Operations", () => {
  let worldStateAddr: PublicKey;
  let materialAuthAddr: PublicKey;
  let currencyCfgAddr: PublicKey;
  let currencyMintAuthAddr: PublicKey;
  let artifactConfigAddr: PublicKey;
  let currencyMintKp: Keypair;
  let sellArtifactMint: Keypair;
  let sellArtifactType: number;

  before(async () => {
    await bootstrapTestEnv();
    const setup = require("./helpers/setup");
    worldStateAddr = setup.worldStateAddr;
    materialAuthAddr = setup.materialAuthAddr;
    currencyCfgAddr = setup.currencyCfgAddr;
    currencyMintAuthAddr = setup.currencyMintAuthAddr;
    artifactConfigAddr = setup.artifactConfigAddr;
    currencyMintKp = setup.currencyMintKeypair;

    await doMultipleForaging(player1, 15, worldStateAddr, materialAuthAddr);
    const craftResult = await forgeArtifactForPlayer(player1, worldStateAddr);
    if (craftResult) {
      sellArtifactMint = craftResult.mint;
      sellArtifactType = craftResult.artifactId;
    } else {
      sellArtifactType = -1;
    }
  });

  it("burns the Artifact and pays the seller in Currency (Sell to Game)", async () => {
    if (sellArtifactType === -1) return;

    const playerCurrencyAta = getAssociatedTokenAddressSync(
      currencyMintKp.publicKey, player1.publicKey, true, TOKEN_2022_PROGRAM_ID
    );
    const existingAta = await anchorProvider.connection.getAccountInfo(playerCurrencyAta);
    if (!existingAta) {
      const createIx = createAssociatedTokenAccountInstruction(
        anchorProvider.wallet.publicKey, playerCurrencyAta, player1.publicKey,
        currencyMintKp.publicKey, TOKEN_2022_PROGRAM_ID
      );
      await anchorProvider.sendAndConfirm(new anchor.web3.Transaction().add(createIx));
    }

    const [artifactMetaPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("artifact_meta"), sellArtifactMint.publicKey.toBuffer()],
      artifactProg.programId
    );
    const playerArtifactAta = getAssociatedTokenAddressSync(
      sellArtifactMint.publicKey, player1.publicKey, false, TOKEN_PROGRAM_ID
    );

    const burnMetaAccounts = [
      { pubkey: findMetadataPda(sellArtifactMint.publicKey), isSigner: false, isWritable: true },
      { pubkey: findMasterEditionPda(sellArtifactMint.publicKey), isSigner: false, isWritable: true },
      { pubkey: METAPLEX_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_INSTRUCTIONS_PUBKEY, isSigner: false, isWritable: false },
    ];

    await tradeProg.methods
      .liquidateArtifact()
      .accountsStrict({
        player: player1.publicKey,
        cpiAuth: tradeCpiPda,
        worldState: worldStateAddr,
        artifactRecord: artifactMetaPda,
        artifactMint: sellArtifactMint.publicKey,
        playerArtifactAta: playerArtifactAta,
        artifactConfig: artifactConfigAddr,
        currencyState: currencyCfgAddr,
        currencyMint: currencyMintKp.publicKey,
        currencyAuth: currencyMintAuthAddr,
        playerCurrencyAta: playerCurrencyAta,
        itemNftProgram: artifactProg.programId,
        magicTokenProgram: currencyProg.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        token2022Program: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .remainingAccounts(burnMetaAccounts)
      .signers([player1])
      .rpc();

    // Check payout
    const currencyBalance = await anchorProvider.connection.getTokenAccountBalance(playerCurrencyAta);
    const expectedAmount = artifactValues[sellArtifactType];
    expect(parseInt(currencyBalance.value.amount)).to.equal(expectedAmount);
  });
});