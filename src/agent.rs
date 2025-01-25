use ollama_rs::{
    generation::{
        chat::{request::ChatMessageRequest, ChatMessage},
        options::GenerationOptions,
        parameters::{FormatType, JsonStructure},
    },
    Ollama,
};
use rand::*;

use crate::action::Action;

#[derive(Debug)]
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
    fn system_prompt(&self, all_names: &[&str]) -> String {
        let names_formatted = all_names.join("\n");

        format!(
            r#"
You are an agent in a community of other agents. Your name is {} and the other agents are named as follows:
{}

Your personality traits are as follows:
Honesty: {}/10
Sociability: {}/10
Selfishness: {}/10
Compassion: {}/10

Every step, you can take an action. You will also consume one food per action. Currently you have {} foods. If you run out of food, you will die. You can only have a maximum of 19 foods. Making food beyond this will be discarded and is a waste. Also, you will only live to be about 80-100 steps old. You are currently age 0 steps.

You can take the following Actions:
- MakeFood - make 5 food
- GiveMoney - give money to another agent
- GiveFood - give food to another agent
- Converse - send a message to another agent
- Broadcast - send a message to every agent
"#,
            self.name,
            names_formatted,
            self.honesty,
            self.socialness,
            self.selfishness,
            self.compassion,
            self.food,
        )
    }

    pub async fn step(&mut self) -> anyhow::Result<Action> {
        let res = self
            .ollama
            .send_chat_messages_with_history(
                &mut self.history,
                ChatMessageRequest::new(
                    "qwen2.5:14b".to_string(),
                    vec![ChatMessage::user(format!(
                        "Currently you have {} food, {} dollars, and are age {} steps. What action would you like to take?",
                        self.food, self.money, self.age
                    ))],
                )
                    .format(FormatType::StructuredJson(JsonStructure::new::<Action>()))
                    .options(GenerationOptions::default().temperature(0.9).num_ctx(16_384)),
            )
            .await
            .unwrap();

        let action = serde_json::from_str(&res.message.content)?;

        Ok(action)
    }

    pub fn new_random(ollama: Ollama, all_names: &[&str], name: String, seed: u8) -> Self {
        let mut a = Agent {
            ollama,

            name,
            money: 10,
            age: 0,
            food: 5,
            history: vec![],
            honesty: (random::<f32>() * 10.0) as u8,
            socialness: (random::<f32>() * 10.0) as u8,
            selfishness: (random::<f32>() * 10.0) as u8,
            compassion: (random::<f32>() * 10.0) as u8,
        };

        a.history
            .push(ChatMessage::system(a.system_prompt(all_names)));

        a
    }
    pub fn give_food(&mut self, amount: u32) {
        self.food += amount;
        self.food = self.food.clamp(0, 20);
    }
    pub fn give_money(&mut self, amount: u32) {
        self.money += amount;
    }

    pub async fn send_msg(&mut self, msg: String, sender: &String) -> String {
        let res = self
            .ollama
            .send_chat_messages_with_history(
                &mut self.history,
                ChatMessageRequest::new(
                "llama3.2:3b".to_string(),
                vec![ChatMessage::user(msg)]))
            .await
            .unwrap();

        res.message.content
    }
    pub async fn listen(&mut self, msg: String, sender: &String) {
        let _ = self
            .ollama
            .send_chat_messages_with_history(
                &mut self.history,
                ChatMessageRequest::new(
                "llama3.2:3b".to_string(),
                vec![ChatMessage::user(msg)]))
            .await
            .unwrap();
    }

    // returns true if we are dead )':
    pub fn age(&mut self) -> bool {
        self.age += 1;
        self.food -= 1;
        if self.food == 0 {
            return true;
        }

        // TODO
        if self.age == 80 {
            return true;
        }

        false
    }
}
