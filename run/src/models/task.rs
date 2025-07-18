use serde::{Deserialize, Serialize};

use crate::models::{convert_to_category, Category};
// On importe la fonction depuis la racine

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    section_id: String,
    pub content: String,
    pub description: String,
    #[serde(skip_deserializing)]  // On ignore ce champ pendant la désérialisation
    #[serde(default)]
    pub category: Option<Category>
}

impl Default for Task {
    fn default() -> Self {
        Task {
            section_id: String::new(),
            content: String::new(),
            description: String::new(),
            category: None,
        }
    }
}

impl Task {
    // Cette méthode sera appelée après la désérialisation
    pub fn post_deserialize(&mut self) {
        self.category = Some(convert_to_category(&self.section_id));
    }
}
