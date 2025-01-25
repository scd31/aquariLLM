pub enum Action {
    MakeFood,
    GiveMoney(GiveInfo),
    GiveFood(GiveInfo),
    Converse(String, usize),
    Broadcast(String, usize),
}

pub struct GiveInfo {
    pub amount: u32,
    pub agent_trading_with: usize,
}
