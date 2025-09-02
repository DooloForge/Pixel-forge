#[turbo::serialize]
pub struct CraftingRecipe {
    pub name: String,
}

#[turbo::serialize]
pub struct CraftingSystem {
    pub recipes: Vec<CraftingRecipe>,
}

impl CraftingSystem {
    pub fn new() -> Self { Self { recipes: vec![] } }
}


