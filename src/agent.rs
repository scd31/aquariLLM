use ollama_rs::{coordinator::Coordinator, generation::{chat::ChatMessage, options::GenerationOptions}};
use rand::*;
use ollama_rs::{
    coordinator::Coordinator,
    generation::{
        chat::{request::ChatMessageRequest, ChatMessage},
        options::GenerationOptions,
        parameters::{FormatType, JsonStructure},
    },
    Ollama,
};

use crate::action::Action;

pub struct Agent {
    pub name: String,
    ollama: Ollama,

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
    pub fn new(ollama: Ollama) -> Self {
        todo!()
    }

    fn system_prompt(&self) -> String {
        format!(
            r#"
You are an agent in a community of other agents.

Your personality traits are as follows:
Honesty: {}/10
Sociability: {}/10
Selfishness: {}/10
Compassion: {}/10

Every step, you can take an action. You will also consume one food per action. Currently you have {} foods. If you run out of food, you will die. Also, you will only live to be about 80-100 steps old. You are currently age 0 steps.
"#,
            self.honesty, self.socialness, self.selfishness, self.compassion, self.food,
        )
    }

    pub async fn step(&mut self) -> anyhow::Result<Action> {
        let res = self
            .ollama
            .send_chat_messages_with_history(
                &mut self.history,
                ChatMessageRequest::new(
                    "llama3.2:3b".to_string(),
                    vec![ChatMessage::user(format!(
                        "Currently you have {} food, {} dollars, and are age {} steps. What action would you like to take?",
                        self.food, self.money, self.age
                    ))],
                )
                .format(FormatType::StructuredJson(JsonStructure::new::<Action>())),
            )
            .await
            .unwrap();

        let action = serde_json::from_str(&res.message.content)?;

        self.age += 1;
        self.food -= 1;
        if self.food == 0 {
            // TODO
            panic!("I am dead");
        }

        Ok(action)
    }

    pub fn new_random(ollama: Ollama, seed: u8) -> Self {
        Agent {
            ollama,

            name: seed.to_string(),
            money: 10,
            age: 0,
            food: 5,
            history: vec![],
            honesty: (random::<f32>() * 10.0) as u8,
            socialness: (random::<f32>() * 10.0) as u8,
            selfishness: (random::<f32>() * 10.0) as u8,
            compassion: (random::<f32>() * 10.0) as u8,
        }
    }
    pub fn give_food(&mut self, amount: u32) {
        self.food += amount;
    }
    pub fn give_money(&mut self, amount: u32) {
        self.money += amount;
    }

    pub fn send_msg(&mut self, msg: String, sender: &String) -> String {
        let msg_back: String = "".to_string();
        msg_back
    }
    pub fn listen(&mut self, msg: String, sender: &String) {
        todo!();
    }
}
