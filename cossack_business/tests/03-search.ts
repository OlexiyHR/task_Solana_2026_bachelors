import * as anchor from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";
import { anchorProvider, resourceMgrProg, forageProg, player1, materialKeypairs, forageCpiPda, cooldownSec, bootstrapTestEnv } from "./helpers/setup";
import { TOKEN_2022_PROGRAM_ID, createMaterialAtas } from "./helpers/utils";

describe("Material Foraging", () => {
  let worldStateAddr: PublicKey;
  let materialAuthAddr: PublicKey;
  let p1MaterialAtas: PublicKey[];

  before(async () => {
    await bootstrapTestEnv();
    const setup = require("./helpers/setup");
    worldStateAddr = setup.worldStateAddr;
    materialAuthAddr = setup.materialAuthAddr;
    p1MaterialAtas = await createMaterialAtas(player1.publicKey);
  });

  it("awards exactly 3 materials on a successful forage", async () => {
    const [playerRecord] = PublicKey.findProgramAddressSync(
      [Buffer.from("player_record"), player1.publicKey.toBuffer()],
      forageProg.programId
    );

    const supplementalAccts = [
      ...materialKeypairs.map((m) => ({ pubkey: m.publicKey, isSigner: false, isWritable: true })),
      ...p1MaterialAtas.map((a) => ({ pubkey: a, isSigner: false, isWritable: true })),
    ];

    await forageProg.methods
      .forageMaterials()
      .accountsStrict({
        player: player1.publicKey,
        record: playerRecord,
        cpiAuth: forageCpiPda,
        worldState: worldStateAddr,
        materialAuth: materialAuthAddr,
        resourceManagerProgram: resourceMgrProg.programId,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .remainingAccounts(supplementalAccts)
      .signers([player1])
      .rpc();

    let materialTotal = 0;
    for (const ata of p1MaterialAtas) {
      const tokenInfo = await anchorProvider.connection.getTokenAccountBalance(ata);
      materialTotal += parseInt(tokenInfo.value.amount);
    }
    expect(materialTotal).to.equal(3);
  });

  it("rejects a forage attempted before the cooldown expires", async () => {
    const [playerRecord] = PublicKey.findProgramAddressSync(
      [Buffer.from("player_record"), player1.publicKey.toBuffer()],
      forageProg.programId
    );

    const supplementalAccts = [
      ...materialKeypairs.map((m) => ({ pubkey: m.publicKey, isSigner: false, isWritable: true })),
      ...p1MaterialAtas.map((a) => ({ pubkey: a, isSigner: false, isWritable: true })),
    ];

    try {
      await forageProg.methods
        .forageMaterials()
        .accountsStrict({
          player: player1.publicKey,
          record: playerRecord,
          cpiAuth: forageCpiPda,
          worldState: worldStateAddr,
          materialAuth: materialAuthAddr,
          resourceManagerProgram: resourceMgrProg.programId,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .remainingAccounts(supplementalAccts)
        .signers([player1])
        .rpc();
      expect.fail("Expected CooldownActive error");
    } catch (err: any) {
      expect(err.toString()).to.include("CooldownActive");
    }
  });
});