use std::io::{stdin, stdout, Write};

use agent::MODEL;
use environment::Environment;
use ollama_rs::{
    generation::chat::{request::ChatMessageRequest, ChatMessage},
    Ollama,
};
use signalbool::{Flag, Signal, SignalBool};

mod action;
mod agent;
mod environment;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ollama = Ollama::default();

    let mut env = Environment::create(ollama, 4);

    println!("Let's meet our cast!");

    for agent in &env.agents {
        println!("{}:", agent.name);
        println!("\tHonesty: {}/10", agent.honesty);
        println!("\tSociability: {}/10", agent.socialness);
        println!("\tSelfishness: {}/10", agent.selfishness);
        println!("\tCompassion: {}/10", agent.compassion);
        println!("\tAbility to make food: {}/10", agent.food_ability);
        println!("\tAbility to make money: {}/10", 10.0 - agent.food_ability);
    }

    let mut sb = SignalBool::new(&[Signal::SIGINT], Flag::Restart)?;

    loop {
        env.run_timestep().await?;

        if sb.caught() {
            sb.reset();

            cli(&mut env).await;

            println!();

            if sb.caught() {
                return Ok(());
            }
        }

        if env.agents.is_empty() {
            break;
        }
    }

    Ok(())
}

async fn cli(env: &mut Environment) -> Option<()> {
    let names: Vec<_> = env.agents.iter().map(|a| a.name.to_string()).collect();

    println!("Who do you want to chat with? [{}]", names.join(", "));
    let mut lines = stdin().lines();
    let mut name;
    let mut agent = loop {
        name = lines.next()?.ok()?;

        if let Some(a) = env
            .agents
            .iter()
            .find(|a| a.name.to_lowercase() == name.to_lowercase())
        {
            break a.to_owned();
        }

        println!("Invalid name");
    };

    println!("Talking to {}", agent.name);

    print!("> ");
    stdout().flush().ok()?;

    for line in lines {
        let line = line.ok()?;

        let res = agent
            .ollama
            .send_chat_messages_with_history(
                &mut agent.history,
                ChatMessageRequest::new(MODEL.to_string(), vec![ChatMessage::user(line)]),
            )
            .await
            .unwrap();

        println!("{}> {}", name, res.message.content);

        print!("> ");
        stdout().flush().ok()?;
    }

    Some(())
}
