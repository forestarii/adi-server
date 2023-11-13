use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct ParamOptions {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateGeneratedValueSchema {
    pub num: Option<i8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateGeneratedValueSchema {
    pub num: Option<i8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateUserSchema {
    pub username: String,
    pub password: String,
}
