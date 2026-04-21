pub const MATERIAL_LIMIT: usize = 6;

/// Artifact blueprints defining required material quantities.
/// Material index order: 0=Wood, 1=Iron, 2=Gold, 3=Leather, 4=Stone, 5=Diamond
pub const BLUEPRINTS: [[u8; MATERIAL_LIMIT]; 4] = [
    [1, 3, 0, 1, 0, 0], // Artifact 0: Saber
    [2, 0, 1, 0, 0, 1], // Artifact 1: Staff
    [0, 2, 1, 4, 0, 0], // Artifact 2: Armor
    [0, 4, 2, 0, 0, 2], // Artifact 3: Bracelet
];