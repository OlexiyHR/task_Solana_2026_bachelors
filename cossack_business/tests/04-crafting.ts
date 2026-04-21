import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey, SYSVAR_RENT_PUBKEY, ComputeBudgetProgram, SystemProgram } from "@solana/web3.js";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import { expect } from "chai";
import { anchorProvider, resourceMgrProg, artifactProg, forgeProg, player1, materialKeypairs, forgeCpiPda, bootstrapTestEnv } from "./helpers/setup";
import { TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, METAPLEX_PROGRAM_ID, BLUEPRINTS,
  findMetadataPda, findMasterEditionPda, doMultipleForaging } from "./helpers/utils";

describe("Artifact Forging (Crafting)", () => {
  let worldStateAddr: PublicKey;
  let materialAuthAddr: PublicKey;
  let artifactConfigAddr: PublicKey;
  let artifactAuthAddr: PublicKey;

  before(async () => {
    await bootstrapTestEnv();
    const setup = require("./helpers/setup");
    worldStateAddr = setup.worldStateAddr;
    materialAuthAddr = setup.materialAuthAddr;
    artifactConfigAddr = setup.artifactConfigAddr;
    artifactAuthAddr = setup.artifactAuthAddr;

    // Forage materials for the player to ensure they have enough to forge an artifact
    await doMultipleForaging(player1, 10, worldStateAddr, materialAuthAddr);
  });

  it("burns the correct materials and mints an Artifact NFT", async () => {
    const balancesBefore: number[] = [];
    for (let idx = 0; idx < 6; idx++) {
      const ata = getAssociatedTokenAddressSync(materialKeypairs[idx].publicKey, player1.publicKey, true, TOKEN_2022_PROGRAM_ID);
      const tokenInfo = await anchorProvider.connection.getTokenAccountBalance(ata);
      balancesBefore.push(parseInt(tokenInfo.value.amount));
    }

    let selectedBlueprint = -1;
    for (let r = 0; r < BLUEPRINTS.length; r++) {
      let canAfford = true;
      for (let idx = 0; idx < 6; idx++) {
        if (balancesBefore[idx] < BLUEPRINTS[r][idx]) { canAfford = false; break; }
      }
      if (canAfford) { selectedBlueprint = r; break; }
    }

    if (selectedBlueprint === -1) {
      console.log("Skipping — random drops were insufficient for any blueprint");
      return;
    }

    const artifactMintKp = Keypair.generate();
    const neededResIds: number[] = [];
    for (let idx = 0; idx < 6; idx++) {
      if (BLUEPRINTS[selectedBlueprint][idx] > 0) neededResIds.push(idx);
    }

    const resAccountEntries = neededResIds.flatMap((rid) => {
      const mintPk = materialKeypairs[rid].publicKey;
      const ataPk = getAssociatedTokenAddressSync(mintPk, player1.publicKey, true, TOKEN_2022_PROGRAM_ID);
      return [
        { pubkey: mintPk, isSigner: false, isWritable: true },
        { pubkey: ataPk, isSigner: false, isWritable: true },
      ];
    });

    const [artifactMetaPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("artifact_meta"), artifactMintKp.publicKey.toBuffer()],
      artifactProg.programId
    );
    const playerNftAta = getAssociatedTokenAddressSync(artifactMintKp.publicKey, player1.publicKey, false, TOKEN_PROGRAM_ID);
    const metadataAddr = findMetadataPda(artifactMintKp.publicKey);
    const masterEdAddr = findMasterEditionPda(artifactMintKp.publicKey);

    const nftAccountEntries = [
      { pubkey: artifactMintKp.publicKey, isSigner: true, isWritable: true },
      { pubkey: playerNftAta, isSigner: false, isWritable: true },
      { pubkey: metadataAddr, isSigner: false, isWritable: true },
      { pubkey: masterEdAddr, isSigner: false, isWritable: true },
      { pubkey: METAPLEX_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: artifactMetaPda, isSigner: false, isWritable: true },
      { pubkey: artifactConfigAddr, isSigner: false, isWritable: false },
      { pubkey: artifactAuthAddr, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    ];

    await forgeProg.methods
      // We keep Buffer.from here because it maps to Vec<u8> in Rust
      .forgeArtifact(selectedBlueprint, Buffer.from(neededResIds)) 
      .accountsStrict({
        player: player1.publicKey,
        cpiAuth: forgeCpiPda,
        worldState: worldStateAddr,
        resourceManagerProgram: resourceMgrProg.programId,
        itemNftProgram: artifactProg.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        token2022Program: TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .remainingAccounts([...resAccountEntries, ...nftAccountEntries])
      .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 })])
      .signers([player1, artifactMintKp])
      .rpc();

    // Verify that the NFT was successfully minted
    const nftBalance = await anchorProvider.connection.getTokenAccountBalance(playerNftAta);
    expect(parseInt(nftBalance.value.amount)).to.equal(1);

    // Verify that the correct amount of materials was burned
    for (let idx = 0; idx < 6; idx++) {
      const ata = getAssociatedTokenAddressSync(materialKeypairs[idx].publicKey, player1.publicKey, true, TOKEN_2022_PROGRAM_ID);
      const tokenInfo = await anchorProvider.connection.getTokenAccountBalance(ata);
      expect(parseInt(tokenInfo.value.amount)).to.equal(balancesBefore[idx] - BLUEPRINTS[selectedBlueprint][idx]);
    }
  });
});