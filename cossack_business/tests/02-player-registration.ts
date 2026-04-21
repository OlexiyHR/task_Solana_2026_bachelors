import * as anchor from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";
import { forageProg, player1, player2, bootstrapTestEnv } from "./helpers/setup";

describe("Player Onboarding", () => {
  before(async () => {
    await bootstrapTestEnv();
  });

  it("creates a player record for wallet #1", async () => {
    const [recordPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("player_record"), player1.publicKey.toBuffer()],
      forageProg.programId
    );

    await forageProg.methods
      .onboardPlayer()
      .accountsStrict({
        player: player1.publicKey,
        record: recordPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([player1])
      .rpc();

    const data = await forageProg.account.playerRecord.fetch(recordPda);
    expect(data.owner.toBase58()).to.equal(player1.publicKey.toBase58());
    expect(data.lastForageTime.toNumber()).to.equal(0);
  });

  it("creates a player record for wallet #2", async () => {
    const [recordPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("player_record"), player2.publicKey.toBuffer()],
      forageProg.programId
    );

    await forageProg.methods
      .onboardPlayer()
      .accountsStrict({
        player: player2.publicKey,
        record: recordPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([player2])
      .rpc();

    const data = await forageProg.account.playerRecord.fetch(recordPda);
    expect(data.owner.toBase58()).to.equal(player2.publicKey.toBase58());
  });
});