use environment::Environment;
use ollama_rs::Ollama;
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
            sb.reset(); // TODO

            loop {
                if sb.caught() {
                    return Ok(());
                }
            }
        }

        if env.agents.is_empty() {
            break;
        }
    }

    Ok(())
}
