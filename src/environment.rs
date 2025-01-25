use crate::{action::ConverseInfo, agent::Agent};
use ollama_rs::Ollama;

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
            new_env
                .agents
                .push(Agent::new_random(ollama.clone(), i as u8));
        }
        new_env
    }
    pub async fn run_timestep(&mut self) -> anyhow::Result<()> {
        let mut actions = Vec::with_capacity(self.agents.len());
        for agent in self.agents.iter_mut() {
            actions.push(agent.step().await?);
        }

        for (i, action) in actions.into_iter().enumerate() {
            match action {
                crate::action::Action::MakeFood => todo!(),
                crate::action::Action::GiveMoney(give_info) => {
                    self.agents[give_info.agent_trading_with].give_money(give_info.amount);
                }
                crate::action::Action::GiveFood(give_info) => {
                    self.agents[give_info.agent_trading_with].give_food(give_info.amount);
                }
                crate::action::Action::Converse(ConverseInfo {
                    directed_at,
                    message,
                }) => {
                    let index = 0; // TODO need to figure this out
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
        }

        Ok(())
    }
}
