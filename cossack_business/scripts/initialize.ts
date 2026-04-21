import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ResourceManager } from "../target/types/resource_manager";
import { MagicToken } from "../target/types/magic_token";
import { ItemNft } from "../target/types/item_nft";
import { Search } from "../target/types/search";
import { Crafting } from "../target/types/crafting";
import { Marketplace } from "../target/types/marketplace";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";

async function main() {
  const anchorProvider = anchor.AnchorProvider.env();
  anchor.setProvider(anchorProvider);

  // Grab handles for each deployed program
  const resourceMgr = anchor.workspace.resourceManager as Program<ResourceManager>;
  const currencyProg = anchor.workspace.magicToken as Program<MagicToken>;
  const artifactProg = anchor.workspace.itemNft as Program<ItemNft>;
  const forageProg = anchor.workspace.search as Program<Search>;
  const forgeProg = anchor.workspace.crafting as Program<Crafting>;
  const tradeProg = anchor.workspace.marketplace as Program<Marketplace>;

  const deployer = anchorProvider.wallet;
  console.log("Deployer wallet:", deployer.publicKey.toBase58());

  // ---------- Game parameters ----------
  const artifactValues = [100, 150, 200, 300];
  const dropChances = [30, 25, 20, 12, 10, 3];
  const cooldownSec = 60;
  const materialNames = ["Wood", "Iron", "Gold", "Leather", "Stone", "Diamond"];
  const materialTickers = ["WOOD", "IRON", "GOLD", "LTHR", "STN", "DMND"];

  // ---------- Derive every PDA ----------
  const [worldStateAddr] = PublicKey.findProgramAddressSync(
    [Buffer.from("world_state")],
    resourceMgr.programId
  );
  const [materialAuthAddr] = PublicKey.findProgramAddressSync(
    [Buffer.from("material_auth")],
    resourceMgr.programId
  );

  const [currencyCfgAddr] = PublicKey.findProgramAddressSync(
    [Buffer.from("currency_state")],
    currencyProg.programId
  );
  const [currencyMintAuthAddr] = PublicKey.findProgramAddressSync(
    [Buffer.from("currency_auth")],
    currencyProg.programId
  );

  const [artifactConfigAddr] = PublicKey.findProgramAddressSync(
    [Buffer.from("artifact_config")],
    artifactProg.programId
  );
  const [artifactAuthAddr] = PublicKey.findProgramAddressSync(
    [Buffer.from("artifact_auth")],
    artifactProg.programId
  );

  // ── Step 1 — WorldState ──────────────────────────────────────────────
  console.log("\nStep 1: Creating WorldState...");
  try {
    await resourceMgr.methods
      .initWorld(
        artifactValues.map((v) => new anchor.BN(v)),
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
    console.log("WorldState PDA:", worldStateAddr.toBase58());
  } catch (e: any) {
    if (e.toString().includes("already in use")) {
      console.log("WorldState already exists — skipping.");
    } else {
      throw e;
    }
  }

  // ── Step 2 — Materials (6 Token-2022 mints) ─────────────────────
  console.log("\nStep 2: Registering material mints...");
  const materialKeypairs: Keypair[] = [];
  for (let ri = 0; ri < 6; ri++) {
    const mintKp = Keypair.generate();
    materialKeypairs.push(mintKp);
    try {
      await resourceMgr.methods
        .registerMaterial(
          ri,
          materialNames[ri],
          materialTickers[ri],
          `https://cossack.game/material/${ri}.json`
        )
        .accountsStrict({
          authority: deployer.publicKey,
          worldState: worldStateAddr,
          mint: mintKp.publicKey,
          materialAuth: materialAuthAddr,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([mintKp])
        .rpc();
      console.log(`Material ${ri} (${materialTickers[ri]}): ${mintKp.publicKey.toBase58()}`);
    } catch (e: any) {
      console.log(`Material ${ri} error:`, e.message?.slice(0, 80));
    }
  }

  // ── Step 3 — Currency mint ─────────────────────────────────────────
  console.log("\nStep 3: Creating Currency mint...");
  const currencyMintKp = Keypair.generate();
  try {
    await currencyProg.methods
      .initCurrency(
        "Cossack Coin",
        "COSSACK",
        "https://cossack.game/coin.json",
        tradeProg.programId
      )
      .accountsStrict({
        admin: deployer.publicKey,
        config: currencyCfgAddr,
        mint: currencyMintKp.publicKey,
        mintAuthority: currencyMintAuthAddr,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([currencyMintKp])
      .rpc();
    console.log("Currency mint:", currencyMintKp.publicKey.toBase58());
  } catch (e: any) {
    if (e.toString().includes("already in use")) {
      console.log("   Currency already exists — skipping.");
    } else {
      throw e;
    }
  }

  // ── Step 4 — Artifact configuration ───────────────────────────────────
  console.log("\nStep 4: Creating Artifact config...");
  try {
    await artifactProg.methods
      .initArtifactSystem(forgeProg.programId, tradeProg.programId)
      .accountsStrict({
        admin: deployer.publicKey,
        config: artifactConfigAddr,
        nftAuthority: artifactAuthAddr,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("   ArtifactConfig PDA:", artifactConfigAddr.toBase58());
  } catch (e: any) {
    if (e.toString().includes("already in use")) {
      console.log("   Artifact config already exists — skipping.");
    } else {
      throw e;
    }
  }

  // ── Summary ──────────────────────────────────────────────────────────
  console.log("\n=== Initialization complete ===");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});