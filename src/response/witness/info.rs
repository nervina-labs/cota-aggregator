use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct IssuerInfo {
    pub name:        String,
    pub avatar:      String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct ClassInfo {
    pub name:           String,
    pub symbol:         String,
    pub description:    String,
    pub image:          String,
    pub audio:          String,
    pub video:          String,
    pub model:          String,
    pub characteristic: String,
    pub properties:     String,
}
