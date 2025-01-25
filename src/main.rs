use environment::Environment;
use ollama_rs::Ollama;

mod action;
mod agent;
mod environment;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ollama = Ollama::default();

    let mut env = Environment::create(ollama, 2);

    loop {
        env.run_timestep().await?;

        if env.agents.is_empty() {
            break;
        }
    }

    Ok(())
}
