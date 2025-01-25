use faker_rand::en_us::names::FirstName;
use ollama_rs::{
    generation::{
        chat::{request::ChatMessageRequest, ChatMessage},
        options::GenerationOptions,
        parameters::{FormatType, JsonStructure},
    },
    Ollama,
};
use rand::*;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::action::LlmAction;

#[derive(Debug)]
pub struct Agent {
    pub name: String,
    ollama: Ollama,

    pub money: u32,
    pub age: u32,
    pub food: u32,
    pub history: Vec<ChatMessage>,

    // attributes (0-10)
    pub honesty: f32,
    pub socialness: f32,
    pub selfishness: f32,
    pub compassion: f32,
    pub food_ability: f32,
}

impl Agent {
    fn system_prompt(&self, all_names: &[String]) -> String {
        let names_formatted = all_names.join("\n");
        let money_ability = 10.0 - self.food_ability;

        format!(
            r#"
You are a person in a virtual community of other people. Your name is {} and the other people are named as follows:
{}

Your personality traits are as follows:
Honesty: {}/10
Sociability: {}/10
Selfishness: {}/10
Compassion: {}/10
Ability to make food: {}/10
Ability to make money: {}/10

Every step, you can take an action. You will also consume one food per action. Currently you have {} foods. If you run out of food, you will die. You can only have a maximum of 19 foods. Making food beyond this will be discarded and is a waste. Wasting food is VERY BAD. Also, you will only live to be about 80-100 steps old. You are currently age 0 steps.

If you want to trade, use messages to try to set up a deal with another person. Then you can each give food/money to eacho other. Keep in mind the other person can always fall through on their end of the deal!

You can take the following Actions:
- Work - get {} money for doing work
- MakeFood(amount) - Make {} food
- GiveMoney(who_to_interact_with, amount) - give money to another person
- GiveFood(who_to_interact_with, amount) - give food to another person
- Converse(who_to_interact_with, message) - send a message to a single other person
- Broadcast(message) - send a message to every person
- Reproduce(who_to_interact_with) - have a baby with another person
"#,
            self.name,
            names_formatted,
            self.honesty,
            self.socialness,
            self.selfishness,
            self.compassion,
            self.food_ability,
            money_ability,
            self.food,
            money_ability,
            self.food_ability,
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
            food_ability: thread_rng().gen_range(0.0..=10.0),
        };

        a.history
            .push(ChatMessage::system(a.system_prompt(all_names)));

        a
    }

    pub fn give_food(&mut self, amount: u32, sender: &String) {
        self.food += amount;
        self.food = self.food.clamp(0, 20);
        self.history.push(ChatMessage::system(format!("You have been given {} food by {}", amount, sender)));
    }

    pub fn give_money(&mut self, amount: u32, sender: &String) {
        self.money += amount;
        self.history.push(ChatMessage::system(format!("You have been given ${} by {}", amount, sender)));
    }

    pub fn make_food(&mut self) {
        self.food += self.food_ability as u32;
        self.food = self.food.clamp(0, 20);
    }

    pub fn work(&mut self) {
        self.money += 10 - self.food_ability as u32;
    }

    pub async fn send_msg(&mut self, msg: String, sender: &String) -> String {
        println!("[DEBUG] {} -> {}: {}", sender, self.name, msg);

        let res = self
            .ollama
            .send_chat_messages_with_history(
                &mut self.history,
                ChatMessageRequest::new(
                "llama3.2:3b".to_string(),
                vec![ChatMessage::user(format!(r#"{} has decided to chat! They said '{msg}' What would you like to say to them?"#, sender)
                )]).format(FormatType::StructuredJson(JsonStructure::new::<MessageReply>())))
            .await
            .unwrap();

        let msg: MessageReply = serde_json::from_str(&res.message.content).unwrap();

        println!("[DEBUG] {} -> {}: {}", self.name, sender, msg.message);

        msg.message
    }

    pub async fn listen(&mut self, msg: String, sender: &String) {
        self.history.push(ChatMessage::user(format!(
            r#"{} has responded! They said '{msg}'"#,
            sender
        )));
    }

    pub async fn propose(&mut self, msg: String, sender: &String) -> anyhow::Result<bool> {
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

    pub fn reproduce(
        &self,
        honesty: f32,
        socialness: f32,
        selfishness: f32,
        compassion: f32,
        food_ability: f32,
        all_names: &[String],
    ) -> Agent {
        let my_weight = random::<f32>();
        let other_weight = 1.0 - my_weight;

        let new_honesty = honesty * other_weight + self.honesty * my_weight;
        let new_socialness = socialness * other_weight + self.socialness * my_weight;
        let new_selfishness = selfishness * other_weight + self.selfishness * my_weight;
        let new_compassion = compassion * other_weight + self.compassion * my_weight;
        let new_food_ability = food_ability * other_weight + self.food_ability * my_weight;

        let mut a = Agent {
            ollama: self.ollama.clone(),

            name: random::<FirstName>().to_string(),
            money: 10,
            age: 0,
            food: 5,
            history: vec![],
            honesty: new_honesty,
            socialness: new_socialness,
            selfishness: new_selfishness,
            compassion: new_compassion,
            food_ability: new_food_ability,
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

#[derive(JsonSchema, Deserialize, Debug)]
struct MessageReply {
    message: String,
}
