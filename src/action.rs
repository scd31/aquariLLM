use ollama_rs::generation::parameters::JsonSchema;
use serde::Deserialize;

#[derive(JsonSchema, Deserialize, Debug)]
pub struct LlmAction {
    thinking: String,
    pub action: Action,
    pub args: ActionArgs,
}

#[derive(JsonSchema, Deserialize, Debug)]
pub enum Action {
    Work,
    MakeFood,
    GiveMoney,
    GiveFood,
    Converse,
    Broadcast,
}

#[derive(JsonSchema, Deserialize, Debug)]
pub struct ActionArgs {
    pub who_to_interact_with: Option<String>,
    pub amount: Option<u32>,
    pub message: Option<String>,
}
