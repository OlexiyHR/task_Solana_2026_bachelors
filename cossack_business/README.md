# Cossack Business — Solana On-Chain Game Ecosystem

A fully on-chain gaming ecosystem built on Solana using the Anchor framework. Players can forage for materials, forge unique Artifact NFTs, and trade them on a decentralized marketplace for an in-game Currency.

## Deployed Program IDs (Devnet)

| Program | Devnet Address |
|---------|---------|
| `resource_manager` | `8xTTGum6kEZtbn6f6vVu3go2YWLwc9r2mbvmRKY23aCj` |
| `magic_token` (Currency) | `7dXKNT6HXVvswbDCtzcLsAaPBa4jsPXnfhF1oP7y3Rgs` |
| `item_nft` (Artifacts) | `HepLhfDjGV28DSL9bhHb9XNRZ2ShqYVtwJ9gFiasgt5s` |
| `search` (Forage) | `6n4ojCjTvG9AGu7oxqQtXPTvw1Zwfn1gEGwgrmEPVrnm` |
| `crafting` (Forge) | `2yezAR5kHwvd8hL89RDnr9UnoHus84uzUuMsvac2ojze` |
| `marketplace` (Trade) | `5imzGijn2kip1VWCQtuLJXNsUc6vVV8UqpZSFSd1v6aD` |

## Microservices Architecture & CPI Flow

Instead of a monolith, the ecosystem is split into six independent Anchor programs. Users can't just mint or burn tokens directly via CLI; everything is routed through Cross-Program Invocations (CPI) and protected by PDA authorities.

```text
┌──────────────┐       ┌──────────────┐
│    search    │──CPI──▶ resource_mgr │ (Mints 3 random materials)
└──────────────┘       └───────┬───────┘
                               │
┌──────────────┐       ┌───────▼───────┐
│   crafting   │──CPI──▶ resource_mgr │ (Burns player materials)
│              │──CPI──▶   item_nft   │ (Mints Metaplex NFT)
└──────────────┘       └──────────────┘
                               
┌──────────────┐       ┌──────────────┐
│ marketplace  │──CPI──▶   item_nft   │ (Burns NFT Artifact)
│              │──CPI──▶ magic_token  │ (Mints MagicToken payout)
└──────────────┘       └──────────────┘
```

## Security & Access Control

1. **CPI Gating:** Every inter-program call requires a specific caller_authority PDA signature. If the seeds don't match the expected caller program, the transaction drops.
2. **No Direct Mint/Burn:** SPL Token-2022 mint authorities belong to the resource_manager and magic_token PDAs. Cheating the system via terminal is impossible.
3. **On-Chain Cooldowns:** The 60-second foraging cooldown is baked into the player's PDA using the on-chain Clock sysvar.

## Development & Deployment

### Prerequisites
- Solana CLI >= 1.18
- Anchor CLI >= 0.32.1
- Node.js & Yarn

### Setup & Testing
The test suite ensures 100% coverage, executing a full local validator clone of Metaplex.

```bash
yarn install
anchor build
anchor test
```

### Devnet Deployment
I've included a bash script that handles the compilation, Devnet cluster deployment, and automatic on-chain state initialization.

```bash
yarn deploy:devnet
```

## Client Interaction Examples

Here is how the client-side interaction looks using the Anchor TS library:

### 1. Forage for Resources (Search Program)

```typescript
await searchProgram.methods
  .searchResources()
  .accountsStrict({
    player: wallet.publicKey,
    playerAccount: playerPda,
    callerAuthority: searchCallerAuth,
    gameConfig: gameConfigPda,
    mintAuthority: mintAuthorityPda,
    resourceManagerProgram: resourceManagerProgram.programId,
    tokenProgram: TOKEN_2022_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  })
  .remainingAccounts(resourceMintAndAtaAccounts)
  .rpc();
```

### 2. Craft an Artifact NFT (Crafting Program)

```typescript
await craftingProgram.methods
  .craftItem(0, Buffer.from(neededResourceIds))
  .accountsStrict({
    player: wallet.publicKey,
    callerAuthority: craftCallerAuth,
    gameConfig: gameConfigPda,
    resourceManagerProgram: resourceManagerProgram.programId,
    itemNftProgram: itemNftProgram.programId,
    tokenProgram: TOKEN_PROGRAM_ID,
    token2022Program: TOKEN_2022_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  })
  .remainingAccounts([...resourceAccounts, ...nftAccounts])
  .signers([nftMintKeypair])
  .rpc();
```

### 3. Sell Artifact for MagicToken (Marketplace Program)

```typescript
await marketplaceProgram.methods
  .sellItemToGame()
  .accountsStrict({
    seller: wallet.publicKey,
    itemMetadata: itemMetadataPda,
    nftMint: nftMintKeypair.publicKey,
    sellerNftAccount: sellerNftAta,
    magicTokenMint: magicMintKp.publicKey,
    sellerMagicAccount: sellerMagicAta,
    gameConfig: gameConfigPda,
    nftAuthority: nftAuthorityPda,
    magicMintAuthority: magicMintAuthPda,
    callerAuthority: marketCallerAuth,
    itemNftProgram: itemNftProgram.programId,
    magicTokenProgram: magicTokenProgram.programId,
    tokenProgram: TOKEN_PROGRAM_ID,
    token2022Program: TOKEN_2022_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```