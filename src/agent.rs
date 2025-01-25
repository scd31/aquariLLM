use ollama_rs::{coordinator::Coordinator, generation::{chat::ChatMessage, options::GenerationOptions}};

use crate::action::Action;

pub struct Agent {
    pub name: String,
    money: u32,
    age: u32,
    food: u32,
    history: Vec<ChatMessage>,

    // attributes (0-10)
    honesty: u8,
    socialness: u8,
    selfishness: u8,
    compassion: u8,
}

impl Agent {
    fn system_prompt(&self) -> String {
        format!(
            r#"
You are an agent in a community of other agents.

Your personality traits are as follows:
Honesty: {}/10
Sociability: {}/10
Selfishness: {}/10
Compassion: {}/10

Every step, you can take an action. You will also consume one food per action. Currently you have {} foods. If you run out of food, you will die.
"#,
            self.honesty, self.socialness, self.selfishness, self.compassion, self.food,
        )
    }

    pub fn step(&mut self) -> Action {
        let tools = tool_group![];

        let mut coordinator =
            Coordinator::new_with_tools(ollama, "llama3.2:3b", &mut self.history, tools)
                .options(GenerationOptions::default().num_ctx(16_384));

        Action::MakeFood
    }

    pub fn new_random(seed : u8) -> Self {
        Agent {
            name : seed.to_string(),
            money: 10,
            age: 0,
            food: 5,
            history: vec![],
            honesty: seed % 10 + 1,
            socialness: (seed + 2) % 10 + 1,
            selfishness: (3 * seed - 2 * seed) % 10 + 1,
            compassion: (100 - seed) % 10 + 1,
        }
    }
    pub fn give_food(&mut self, amount : u32) {
        self.food += amount;
    }
    pub fn give_money(&mut self, amount : u32) {
        self.money += amount;
    }

    pub fn send_msg(&mut self, msg : String, sender : &String) -> String {
        let msg_back : String = "".to_string();
        msg_back
    }
    pub fn listen(&mut self, msg : String, sender : &String) {
        todo!();
    }
}
