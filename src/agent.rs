use ollama_rs::generation::chat::ChatMessage;

pub struct Agent {
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

    fn step(&mut self) {
        let tools = tool_group![];

        let mut coordinator =
            Coordinator::new_with_tools(ollama, "llama3.2:3b", &mut self.history, tools)
                .options(GenerationOptions::default().num_ctx(16_384));
    }
}
