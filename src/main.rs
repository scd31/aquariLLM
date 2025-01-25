use agent::Agent;
use ollama_rs::{generation::options::GenerationOptions, Ollama};

mod action;
mod agent;
mod environment;

#[tokio::main]
async fn main() {
    let mut ollama = Ollama::default();

    let mut agent = Agent::new_random(ollama, 0);
    loop {
        dbg!(agent.step().await.unwrap());
    }

    println!("Hello, world!");
}
