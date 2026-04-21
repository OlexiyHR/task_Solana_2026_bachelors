import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey, SYSVAR_RENT_PUBKEY, ComputeBudgetProgram } from "@solana/web3.js";
import { TOKEN_2022_PROGRAM_ID, getAssociatedTokenAddressSync, createAssociatedTokenAccountInstruction,
  ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { anchorProvider, resourceMgrProg, forageProg, forgeProg, artifactProg, deployer, materialKeypairs,
  forageCpiPda, forgeCpiPda, artifactAuthAddr, artifactConfigAddr } from "./setup";

export { TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID };

export const METAPLEX_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

export const BLUEPRINTS = [
  [1, 3, 0, 1, 0, 0], // Saber
  [2, 0, 1, 0, 0, 1], // Staff
  [0, 2, 1, 4, 0, 0], // Armor
  [0, 4, 2, 0, 0, 2], // Bracelet
];

export function findMetadataPda(mint: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      METAPLEX_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
    ],
    METAPLEX_PROGRAM_ID
  )[0];
}

export function findMasterEditionPda(mint: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      METAPLEX_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
      Buffer.from("edition"),
    ],
    METAPLEX_PROGRAM_ID
  )[0];
}

export async function createMaterialAtas(ownerPubkey: PublicKey): Promise<PublicKey[]> {
  const ataAddrs: PublicKey[] = [];
  for (let i = 0; i < 6; i++) {
    const ata = getAssociatedTokenAddressSync(
      materialKeypairs[i].publicKey,
      ownerPubkey,
      true,
      TOKEN_2022_PROGRAM_ID
    );
    ataAddrs.push(ata);
    const existingInfo = await anchorProvider.connection.getAccountInfo(ata);
    if (!existingInfo) {
      const createAtaIx = createAssociatedTokenAccountInstruction(
        anchorProvider.wallet.publicKey,
        ata,
        ownerPubkey,
        materialKeypairs[i].publicKey,
        TOKEN_2022_PROGRAM_ID
      );
      const tx = new anchor.web3.Transaction().add(createAtaIx);
      await anchorProvider.sendAndConfirm(tx);
    }
  }
  return ataAddrs;
}

export async function doMultipleForaging(
  playerKp: Keypair,
  rounds: number,
  worldStateAddr: PublicKey,
  materialAuth: PublicKey,
) {
  await resourceMgrProg.methods
    .setTimer(new anchor.BN(1))
    .accountsStrict({ authority: deployer.publicKey, worldState: worldStateAddr })
    .rpc();

  const [playerRecordPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("player_record"), playerKp.publicKey.toBuffer()],
    forageProg.programId
  );

  const playerTokenAccts: PublicKey[] = [];
  for (let i = 0; i < 6; i++) {
    playerTokenAccts.push(
      getAssociatedTokenAddressSync(
        materialKeypairs[i].publicKey,
        playerKp.publicKey,
        true,
        TOKEN_2022_PROGRAM_ID
      )
    );
  }

  const supplementalAccts = [
    ...materialKeypairs.map((m) => ({
      pubkey: m.publicKey, isSigner: false, isWritable: true,
    })),
    ...playerTokenAccts.map((a) => ({
      pubkey: a, isSigner: false, isWritable: true,
    })),
  ];

  for (let iter = 0; iter < rounds; iter++) {
    await new Promise((resolve) => setTimeout(resolve, 1500));
    try {
      await forageProg.methods
        .forageMaterials()
        .accountsStrict({
          player: playerKp.publicKey,
          record: playerRecordPda,
          cpiAuth: forageCpiPda,
          worldState: worldStateAddr,
          materialAuth: materialAuth,
          resourceManagerProgram: resourceMgrProg.programId,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .remainingAccounts(supplementalAccts)
        .signers([playerKp])
        .rpc();
    } catch {
      await new Promise((resolve) => setTimeout(resolve, 2000));
      await forageProg.methods
        .forageMaterials()
        .accountsStrict({
          player: playerKp.publicKey,
          record: playerRecordPda,
          cpiAuth: forageCpiPda,
          worldState: worldStateAddr,
          materialAuth: materialAuth,
          resourceManagerProgram: resourceMgrProg.programId,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .remainingAccounts(supplementalAccts)
        .signers([playerKp])
        .rpc();
    }
  }

  await resourceMgrProg.methods
    .setTimer(new anchor.BN(2))
    .accountsStrict({ authority: deployer.publicKey, worldState: worldStateAddr })
    .rpc();
}

export async function forgeArtifactForPlayer( playerKp: Keypair, worldStateAddr: PublicKey ): Promise<{ mint: Keypair; artifactId: number } | null> {
  const playerTokenAccts: PublicKey[] = [];
  const balanceAmounts: number[] = [];
  for (let i = 0; i < 6; i++) {
    const ata = getAssociatedTokenAddressSync(materialKeypairs[i].publicKey, playerKp.publicKey, true, TOKEN_2022_PROGRAM_ID);
    playerTokenAccts.push(ata);
    const info = await anchorProvider.connection.getTokenAccountBalance(ata);
    balanceAmounts.push(parseInt(info.value.amount));
  }

  let selectedBlueprint = -1;
  for (let r = 0; r < BLUEPRINTS.length; r++) {
    let canAfford = true;
    for (let i = 0; i < 6; i++) {
      if (balanceAmounts[i] < BLUEPRINTS[r][i]) { canAfford = false; break; }
    }
    if (canAfford) { selectedBlueprint = r; break; }
  }

  if (selectedBlueprint === -1) return null;
  const newArtifactMint = Keypair.generate();

  const neededResIds: number[] = [];
  for (let i = 0; i < 6; i++) {
    if (BLUEPRINTS[selectedBlueprint][i] > 0) neededResIds.push(i);
  }

  const resAccountEntries = neededResIds.flatMap((rid) => {
    const mintPk = materialKeypairs[rid].publicKey;
    const ataPk = getAssociatedTokenAddressSync(mintPk, playerKp.publicKey, true, TOKEN_2022_PROGRAM_ID);
    return [
      { pubkey: mintPk, isSigner: false, isWritable: true },
      { pubkey: ataPk, isSigner: false, isWritable: true },
    ];
  });

  const [artifactMetaPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("artifact_meta"), newArtifactMint.publicKey.toBuffer()],
    artifactProg.programId
  );
  const playerNftAta = getAssociatedTokenAddressSync(
    newArtifactMint.publicKey, playerKp.publicKey, false, TOKEN_PROGRAM_ID
  );
  const metadataAddr = findMetadataPda(newArtifactMint.publicKey);
  const masterEdAddr = findMasterEditionPda(newArtifactMint.publicKey);

  const nftAccountEntries = [
    { pubkey: newArtifactMint.publicKey, isSigner: true, isWritable: true },
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
    .forgeArtifact(selectedBlueprint, Buffer.from(neededResIds))
    .accountsStrict({
      player: playerKp.publicKey,
      cpiAuth: forgeCpiPda,
      worldState: worldStateAddr,
      resourceManagerProgram: resourceMgrProg.programId,
      itemNftProgram: artifactProg.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      token2022Program: TOKEN_2022_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .remainingAccounts([...resAccountEntries, ...nftAccountEntries])
    .preInstructions([
      ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 }),
    ])
    .signers([playerKp, newArtifactMint])
    .rpc();

  return { mint: newArtifactMint, artifactId: selectedBlueprint };
}
