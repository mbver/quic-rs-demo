use serde::{Serialize, Deserialize};

pub const ADMIN_USERNAME: &str = "admin";

#[derive(Debug, Serialize, Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
  pub token: String,
  pub signature: String,
}