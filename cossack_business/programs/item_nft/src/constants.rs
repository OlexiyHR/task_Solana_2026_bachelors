use anchor_lang::prelude::*;

/// Total number of distinct craftable artifacts in the game.
pub const ARTIFACT_TYPES_COUNT: usize = 4;

/// Display names attached to each Artifact NFT via Metaplex metadata.
pub const ARTIFACT_TITLES: [&str; ARTIFACT_TYPES_COUNT] = [
    "Cossack Saber",
    "Elder Staff",
    "Mage Armor",
    "Battle Bracelet",
];

/// Short ticker symbols for each artifact type.
pub const ARTIFACT_TICKERS: [&str; ARTIFACT_TYPES_COUNT] = ["SABER", "STAFF", "ARMOR", "BRACLT"];

/// On-chain address of the Metaplex Token Metadata program.
pub const MPL_TOKEN_METADATA_ID: Pubkey =
    anchor_lang::pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");