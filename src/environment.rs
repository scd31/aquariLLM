use crate::{action::ConverseInfo, agent::Agent};
use ollama_rs::Ollama;

const POTENTIAL_NAMES: [&'static str; 4] = ["Brenda", "Emma", "Stephen", "Basil"];

pub struct Environment {
    time: u32,
    agents: Vec<Agent>,
}

impl Environment {
    pub fn create(ollama: Ollama, num_agents: usize) -> Self {
        let mut new_env = Environment {
            time: 0,
            agents: Vec::with_capacity(num_agents),
        };

        for i in 0..num_agents {
            // TODO
            let name = POTENTIAL_NAMES[i].to_string();

            new_env
                .agents
                .push(Agent::new_random(ollama.clone(), name, i as u8));
        }
        new_env
    }

    pub async fn run_timestep(&mut self) -> anyhow::Result<()> {
        let mut actions = Vec::with_capacity(self.agents.len());
        for agent in self.agents.iter_mut() {
            actions.push(agent.step().await?);
        }

        let mut dead = vec![];

        for (i, action) in actions.into_iter().enumerate() {
            println!("[DEBUG] {} took action {:?}", self.agents[i].name, action);
            dbg!(&self.agents[i]);

            match action {
                crate::action::Action::MakeFood => self.agents[i].food += 5,
                crate::action::Action::GiveMoney(give_info) => {
                    let other_id = self.get_id_from_name(&give_info.agent_trading_with);

                    // todo tell the LLM they're an idiot
                    let other_id = other_id.unwrap();

                    self.agents[other_id].give_money(give_info.amount);
                }
                crate::action::Action::GiveFood(give_info) => {
                    let other_id = self.get_id_from_name(&give_info.agent_trading_with);

                    // todo tell the LLM they're an idiot
                    let other_id = other_id.unwrap();

                    self.agents[other_id].give_food(give_info.amount);
                }
                crate::action::Action::Converse(ConverseInfo {
                    directed_at,
                    message,
                }) => {
                    // TODO tell LLM they're an idiot
                    let index = self.get_id_from_name(&directed_at).unwrap();
                    let name = self.agents[i].name.clone();
                    let msg_back = self.agents[index].send_msg(message, &name);
                    self.agents[i].listen(msg_back, &name);
                }
                crate::action::Action::Broadcast(message) => {
                    let name = self.agents[i].name.clone();
                    for j in 0..self.agents.len() {
                        if i == j {
                            continue;
                        }
                        self.agents[j].listen(message.clone(), &name);
                    }
                }
            }

            if self.agents[i].age() {
                dead.push(i);
            }
        }

        for d in dead.into_iter().rev() {
            self.agents.remove(d);
        }

        Ok(())
    }

    fn get_id_from_name(&self, name: &str) -> Option<usize> {
        self.agents
            .iter()
            .enumerate()
            .find(|(_, a)| a.name == name)
            .map(|(id, _)| id)
    }
}
