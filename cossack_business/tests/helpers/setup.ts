import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ResourceManager } from "../../target/types/resource_manager";
import { MagicToken } from "../../target/types/magic_token";
import { ItemNft } from "../../target/types/item_nft";
import { Search } from "../../target/types/search";
import { Crafting } from "../../target/types/crafting";
import { Marketplace } from "../../target/types/marketplace";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";

// ---------- Provider & program handles ----------
export const anchorProvider = anchor.AnchorProvider.env();
anchor.setProvider(anchorProvider);

export const resourceMgrProg = anchor.workspace
  .resourceManager as Program<ResourceManager>;
export const currencyProg = anchor.workspace.magicToken as Program<MagicToken>;
export const artifactProg = anchor.workspace.itemNft as Program<ItemNft>;
export const forageProg = anchor.workspace.search as Program<Search>;
export const forgeProg = anchor.workspace.crafting as Program<Crafting>;
export const tradeProg = anchor.workspace.marketplace as Program<Marketplace>;

// Deployer wallet exposed by the anchor provider
export const deployer = anchorProvider.wallet;

// ---------- Material configuration ----------
export const materialKeypairs: Keypair[] = [];
export const materialNames = [
  "Wood", "Iron", "Gold", "Leather", "Stone", "Diamond",
];
export const materialTickers = [
  "WOOD", "IRON", "GOLD", "LTHR", "STN", "DMND",
];

// ---------- PDA placeholders (populated in bootstrapTestEnv) ----------
export let currencyMintKeypair: Keypair;
export let worldStateAddr: PublicKey;
export let materialAuthAddr: PublicKey;
export let currencyCfgAddr: PublicKey;
export let currencyMintAuthAddr: PublicKey;
export let artifactConfigAddr: PublicKey;
export let artifactAuthAddr: PublicKey;

// CPI Authority PDAs (cpi_auth)
export const forageCpiPda = PublicKey.findProgramAddressSync(
  [Buffer.from("cpi_auth")],
  forageProg.programId
)[0];
export const forgeCpiPda = PublicKey.findProgramAddressSync(
  [Buffer.from("cpi_auth")],
  forgeProg.programId
)[0];
export const tradeCpiPda = PublicKey.findProgramAddressSync(
  [Buffer.from("cpi_auth")],
  tradeProg.programId
)[0];

// ---------- Test players ----------
export const player1 = Keypair.generate();
export const player2 = Keypair.generate();

// ---------- Game parameters for tests ----------
export const artifactValues = [100, 150, 200, 300];
export const dropChances = [30, 25, 20, 12, 10, 3];
export const cooldownSec = 2; // seconds (low value for faster tests)

let alreadyInitialized = false;

/**
 * One-time bootstrap that funds test wallets, derives PDAs, and generates
 * keypairs for material mints and the currency mint.
 */
export async function bootstrapTestEnv(): Promise<void> {
  if (alreadyInitialized) return;
  alreadyInitialized = true;

  // Airdrop SOL to each test player
  for (const wallet of [player1, player2]) {
    const sig = await anchorProvider.connection.requestAirdrop(
      wallet.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );
    await anchorProvider.connection.confirmTransaction(sig);
  }

  // Derive PDAs matching Rust logic
  [worldStateAddr] = PublicKey.findProgramAddressSync(
    [Buffer.from("world_state")],
    resourceMgrProg.programId
  );
  [materialAuthAddr] = PublicKey.findProgramAddressSync(
    [Buffer.from("material_auth")],
    resourceMgrProg.programId
  );

  [currencyCfgAddr] = PublicKey.findProgramAddressSync(
    [Buffer.from("currency_state")],
    currencyProg.programId
  );
  [currencyMintAuthAddr] = PublicKey.findProgramAddressSync(
    [Buffer.from("currency_auth")],
    currencyProg.programId
  );

  [artifactConfigAddr] = PublicKey.findProgramAddressSync(
    [Buffer.from("artifact_config")],
    artifactProg.programId
  );
  [artifactAuthAddr] = PublicKey.findProgramAddressSync(
    [Buffer.from("artifact_auth")],
    artifactProg.programId
  );

  // Generate keypairs for the 6 material mints
  for (let i = 0; i < 6; i++) {
    materialKeypairs.push(Keypair.generate());
  }
  currencyMintKeypair = Keypair.generate();

  // Re-export mutable values
  exports.currencyMintKeypair = currencyMintKeypair;
  exports.worldStateAddr = worldStateAddr;
  exports.materialAuthAddr = materialAuthAddr;
  exports.currencyCfgAddr = currencyCfgAddr;
  exports.currencyMintAuthAddr = currencyMintAuthAddr;
  exports.artifactConfigAddr = artifactConfigAddr;
  exports.artifactAuthAddr = artifactAuthAddr;
}
