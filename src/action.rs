use ollama_rs::generation::parameters::JsonSchema;
use serde::Deserialize;

/*pub struct RawAction {
    action: ActionName,

}*/

#[derive(JsonSchema, Deserialize, Debug)]
pub enum Action {
    MakeFood,
    GiveMoney(GiveInfo),
    GiveFood(GiveInfo),
    Converse(ConverseInfo),
    Broadcast(String),
}

#[derive(JsonSchema, Deserialize, Debug)]
pub struct GiveInfo {
    pub amount: u32,
    pub agent_trading_with: usize,
}

#[derive(JsonSchema, Deserialize, Debug)]
pub struct ConverseInfo {
    pub directed_at: String,
    pub message: String,
}
