# Cossack Business — Solana On-Chain Game Ecosystem

A fully on-chain gaming ecosystem built on Solana using the Anchor framework. Players can forage for materials, forge unique Artifact NFTs, and trade them on a decentralized marketplace for an in-game Currency.

This project fulfills all requirements for the WhiteBIT "Cossack Business" technical assignment, including strict CPI-gating, PDA authority controls, Token-2022 integrations, and 100% test coverage.

## 🔗 Deployed Program IDs (Devnet)

| Program | Devnet Address |
|---------|---------|
| `resource_manager` | `8xTTGum6kEZtbn6f6vVu3go2YWLwc9r2mbvmRKY23aCj` |
| `magic_token` (Currency) | `7dXKNT6HXVvswbDCtzcLsAaPBa4jsPXnfhF1oP7y3Rgs` |
| `item_nft` (Artifacts) | `HepLhfDjGV28DSL9bhHb9XNRZ2ShqYVtwJ9gFiasgt5s` |
| `search` (Forage) | `6n4ojCjTvG9AGu7oxqQtXPTvw1Zwfn1gEGwgrmEPVrnm` |
| `crafting` (Forge) | `2yezAR5kHwvd8hL89RDnr9UnoHus84uzUuMsvac2ojze` |
| `marketplace` (Trade) | `5imzGijn2kip1VWCQtuLJXNsUc6vVV8UqpZSFSd1v6aD` |

## 🏗️ Microservices Architecture & CPI Flow

The game comprises six independent, highly decoupled Anchor programs. Direct minting/burning of tokens or NFTs by users is strictly prohibited; all asset generation is gated through Cross-Program Invocations (CPI) using PDA authorities.

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

## 🛡️ Security & Access Control

1. **Strict CPI Gating:** Programs utilize `caller_authority` PDAs. A receiving program will only execute if the caller provides a valid PDA seed matching the expected authorized program ID.
2. **No Direct Mint/Burn:** SPL Token-2022 mint authorities are PDAs owned by `resource_manager` and `magic_token`. Standard CLI minting is impossible.
3. **On-Chain Timers:** Foraging cooldowns (60 seconds) are enforced securely on-chain using the `Clock` sysvar stored in the player's PDA.

## 🛠️ Development & Deployment

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
A complete bash script is provided to handle compilation, Devnet cluster deployment, and automatic on-chain PDA initialization.

```bash
yarn deploy:devnet
```

## 💻 Interaction Examples

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
  .remainingAccounts(resourceMintAndAtaAccounts) // 6 mints + 6 ATAs
  .rpc();
```

### 2. Craft an Artifact NFT (Crafting Program)

```typescript
// Craft Recipe 0 (e.g., Cossack Saber)
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