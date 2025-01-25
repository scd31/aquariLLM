use ollama_rs::{
    generation::{
        chat::{request::ChatMessageRequest, ChatMessage},
        options::GenerationOptions,
        parameters::{FormatType, JsonStructure},
    },
    Ollama,
};
use rand::*;
use faker_rand::en_us::names::FirstName;

use crate::action::LlmAction;

#[derive(Debug)]
pub struct Agent {
    pub name: String,
    ollama: Ollama,

    pub money: u32,
    pub age: u32,
    pub food: u32,
    history: Vec<ChatMessage>,

    // attributes (0-10)
    pub honesty: f32,
    pub socialness: f32,
    pub selfishness: f32,
    pub compassion: f32,
}

impl Agent {
    fn system_prompt(&self, all_names: &[String]) -> String {
        let names_formatted = all_names.join("\n");

        format!(
            r#"
You are a person in a virtual community of other people. Your name is {} and the other agents are named as follows:
{}

Your personality traits are as follows:
Honesty: {}/10
Sociability: {}/10
Selfishness: {}/10
Compassion: {}/10

Every step, you can take an action. You will also consume one food per action. Currently you have {} foods. If you run out of food, you will die. You can only have a maximum of 19 foods. Making food beyond this will be discarded and is a waste. Wasting food is VERY BAD. Also, you will only live to be about 80-100 steps old. You are currently age 0 steps.

You can take the following Actions:
- Work - get 5 money for doing work
- MakeFood(amount) - exchange money for food at a 1:1 ratio
- GiveMoney(who_to_interact_with, amount) - give money to another agent
- GiveFood(who_to_interact_with, amount) - give food to another agent
- Converse(who_to_interact_with, message) - send a message to a single other agent
- Broadcast(message) - send a message to every agent
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

    pub async fn step(&mut self) -> anyhow::Result<LlmAction> {
        let res = self
            .ollama
            .send_chat_messages_with_history(
                &mut self.history,
                ChatMessageRequest::new(
                    "llama3.2:3b".to_string(),
                    vec![ChatMessage::user(format!(
                        "Currently you have {} food (max 19, dead at 0), {} dollars, and are age {} steps. What action would you like to take?",
                        self.food, self.money, self.age
                    ))],
                )
                    .format(FormatType::StructuredJson(JsonStructure::new::<LlmAction>()))
                    .options(GenerationOptions::default().temperature(0.9).num_ctx(16_384)),
            )
            .await
            .unwrap();

        dbg!(&res.message.content);

        let action = serde_json::from_str(&res.message.content)?;

        Ok(action)
    }

    pub fn new_random(ollama: Ollama, all_names: &[String], name: String) -> Self {
        let mut a = Agent {
            ollama,

            name,
            money: 10,
            age: 0,
            food: 5,
            history: vec![],
            honesty: random::<f32>() * 10.0,
            socialness: random::<f32>() * 10.0,
            selfishness: random::<f32>() * 10.0,
            compassion: random::<f32>() * 10.0,
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
                vec![ChatMessage::user(format!(r#"{} has decided to chat! They said '{msg}' What would you like to say to them?"#, sender)
                )]))
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
                    vec![ChatMessage::user(format!(
                        r#"{} has responded! They said '{msg}'"#,
                        sender
                    ))],
                ),
            )
            .await
            .unwrap();
    }

    pub async fn propose (&mut self, msg : String, sender: &String) ->anyhow::Result<bool> {
        let res = self
            .ollama
            .send_chat_messages_with_history(
                &mut self.history,
                ChatMessageRequest::new(
                "llama3.2:3b".to_string(),
                vec![ChatMessage::user(format!(r#"{} has proposed to reproduce! They said '{msg}' Do you accept? Respond true or false."#, sender)
                )])
                .format(FormatType::StructuredJson(JsonStructure::new::<bool>()))
            )
            .await
            .unwrap();

        let action = serde_json::from_str(&res.message.content)?;

        Ok(action)
    }

    pub fn reproduce (&self, honesty : f32, socialness : f32, selfishness : f32, compassion : f32, all_names: &[String]) -> Agent {
        let my_weight = random::<f32>();
        let other_weight = 1.0 - my_weight;

        let new_honesty = honesty * other_weight + self.honesty * my_weight;
        let new_socialness = socialness * other_weight + self.socialness * my_weight;
        let new_selfishness = selfishness * other_weight + self.selfishness * my_weight;
        let new_compassion = compassion * other_weight + self.compassion * my_weight;

        let mut a = Agent {
            ollama : self.ollama.clone(),

            name : random::<FirstName>().to_string(),
            money: 10,
            age: 0,
            food: 5,
            history: vec![],
            honesty: new_honesty,
            socialness: new_socialness,
            selfishness: new_selfishness,
            compassion: new_compassion,
        };

        let mut new_names = all_names.to_vec();
        new_names.push(a.name.clone());

        a.history
            .push(ChatMessage::system(a.system_prompt(&new_names)));

        a

    }

    // returns true if we are dead )':
    pub fn age(&mut self) -> bool {
        self.age += 1;
        if self.food == 0 {
            return true;
        }
        self.food -= 1;

        // TODO
        if self.age == 80 {
            return true;
        }

        false
    }
}
