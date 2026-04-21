import * as anchor from "@coral-xyz/anchor";
import { SystemProgram } from "@solana/web3.js";
import { expect } from "chai";
import { anchorProvider, resourceMgrProg, currencyProg, artifactProg, forageProg, forgeProg, tradeProg, deployer,
  materialKeypairs, materialNames, materialTickers, artifactValues, dropChances, cooldownSec, bootstrapTestEnv } from "./helpers/setup";
import { TOKEN_2022_PROGRAM_ID } from "./helpers/utils";

describe("Ecosystem Bootstrap & Admin Operations", () => {
  let worldStateAddr: anchor.web3.PublicKey;
  let materialAuthAddr: anchor.web3.PublicKey;
  let currencyCfgAddr: anchor.web3.PublicKey;
  let currencyMintAuthAddr: anchor.web3.PublicKey;
  let artifactConfigAddr: anchor.web3.PublicKey;
  let artifactAuthAddr: anchor.web3.PublicKey;

  before(async () => {
    await bootstrapTestEnv();
    const setup = require("./helpers/setup");
    worldStateAddr = setup.worldStateAddr;
    materialAuthAddr = setup.materialAuthAddr;
    currencyCfgAddr = setup.currencyCfgAddr;
    currencyMintAuthAddr = setup.currencyMintAuthAddr;
    artifactConfigAddr = setup.artifactConfigAddr;
    artifactAuthAddr = setup.artifactAuthAddr;
  });

  it("creates the WorldState account with correct parameters", async () => {
    await resourceMgrProg.methods
      .initWorld(
        artifactValues.map((p) => new anchor.BN(p)),
        dropChances,
        new anchor.BN(cooldownSec),
        forageProg.programId,
        forgeProg.programId,
        tradeProg.programId
      )
      .accountsStrict({
        authority: deployer.publicKey,
        worldState: worldStateAddr,
        materialAuth: materialAuthAddr,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const storedCfg = await resourceMgrProg.account.worldState.fetch(worldStateAddr);
    expect(storedCfg.authority.toBase58()).to.equal(deployer.publicKey.toBase58());
    expect(storedCfg.forageCooldown.toNumber()).to.equal(cooldownSec);
    expect(storedCfg.registeredMaterials).to.equal(0);
  });

  it("registers all six material mints", async () => {
    for (let idx = 0; idx < 6; idx++) {
      await resourceMgrProg.methods
        .registerMaterial(idx, materialNames[idx], materialTickers[idx], `https://cossack.game/material/${idx}.json`)
        .accountsStrict({
          authority: deployer.publicKey,
          worldState: worldStateAddr,
          mint: materialKeypairs[idx].publicKey,
          materialAuth: materialAuthAddr,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([materialKeypairs[idx]])
        .rpc();
    }
    const storedCfg = await resourceMgrProg.account.worldState.fetch(worldStateAddr);
    expect(storedCfg.registeredMaterials).to.equal(6);
  });

  it("initializes the in-game Currency", async () => {
    const setup = require("./helpers/setup");
    await currencyProg.methods
      .initCurrency("Cossack Coin", "COSSACK", "https://cossack.game/coin.json", tradeProg.programId)
      .accountsStrict({
        admin: deployer.publicKey,
        config: currencyCfgAddr,
        mint: setup.currencyMintKeypair.publicKey,
        mintAuthority: currencyMintAuthAddr,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([setup.currencyMintKeypair])
      .rpc();
    
    const cfg = await currencyProg.account.currencyState.fetch(currencyCfgAddr);
    expect(cfg.tokenMint.toBase58()).to.equal(setup.currencyMintKeypair.publicKey.toBase58());
  });

  it("configures the Artifact NFT system", async () => {
    await artifactProg.methods
      .initArtifactSystem(forgeProg.programId, tradeProg.programId)
      .accountsStrict({
        admin: deployer.publicKey,
        config: artifactConfigAddr,
        nftAuthority: artifactAuthAddr,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  });
});
