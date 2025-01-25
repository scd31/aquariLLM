use ollama_rs::{generation::options::GenerationOptions, Ollama};

mod action;
mod agent;

fn main() {
    let mut ollama = Ollama::default();

    println!("Hello, world!");
}
