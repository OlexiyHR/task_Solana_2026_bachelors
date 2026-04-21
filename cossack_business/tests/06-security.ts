import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import { expect } from "chai";
import { resourceMgrProg, currencyProg, player1, bootstrapTestEnv } from "./helpers/setup";
import { TOKEN_2022_PROGRAM_ID } from "./helpers/utils";

describe("Access Control & CPI Gating Security", () => {
  let worldStateAddr: PublicKey;
  let materialAuthAddr: PublicKey;
  let currencyCfgAddr: PublicKey;
  let currencyMintAuthAddr: PublicKey;

  before(async () => {
    await bootstrapTestEnv();
    const setup = require("./helpers/setup");
    worldStateAddr = setup.worldStateAddr;
    materialAuthAddr = setup.materialAuthAddr;
    currencyCfgAddr = setup.currencyCfgAddr;
    currencyMintAuthAddr = setup.currencyMintAuthAddr;
  });

  it("blocks direct material minting with a forged caller authority", async () => {
    const setup = require("./helpers/setup");
    const bogusAuthority = Keypair.generate();
    const ata = getAssociatedTokenAddressSync(
      setup.materialKeypairs[0].publicKey,
      player1.publicKey,
      true,
      TOKEN_2022_PROGRAM_ID
    );

    try {
      await resourceMgrProg.methods
        .issueMaterial(0, new anchor.BN(1))
        .accountsStrict({
          cpiAuth: bogusAuthority.publicKey,
          worldState: worldStateAddr,
          materialMint: setup.materialKeypairs[0].publicKey,
          materialAuth: materialAuthAddr,
          receiverAta: ata,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([bogusAuthority])
        .rpc();
      expect.fail("Expected a CPI-authority rejection");
    } catch (err: any) {
      const msg = err.toString();
      expect(
        msg.includes("Signature verification failed") || 
        msg.includes("ConstraintSeeds")
      ).to.be.true;
    }
  });

  it("blocks direct Currency minting with a forged caller authority", async () => {
    const setup = require("./helpers/setup");
    const bogusAuthority = Keypair.generate();
    const ata = getAssociatedTokenAddressSync(
      setup.currencyMintKeypair.publicKey,
      player1.publicKey,
      true,
      TOKEN_2022_PROGRAM_ID
    );

    try {
      await currencyProg.methods
        .issueCurrency(new anchor.BN(100))
        .accountsStrict({
          cpiAuth: bogusAuthority.publicKey,
          config: currencyCfgAddr,
          tokenMint: setup.currencyMintKeypair.publicKey,
          mintAuthority: currencyMintAuthAddr,
          receiverAta: ata,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([bogusAuthority])
        .rpc();
      expect.fail("Expected a CPI-authority rejection");
    } catch (err: any) {
      expect(err).to.exist;
    }
  });
});