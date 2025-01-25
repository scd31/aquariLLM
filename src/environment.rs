use crate::{action::Action, agent::Agent};
use ollama_rs::Ollama;

const POTENTIAL_NAMES: [&str; 4] = ["Brenda", "Emma", "Stephen", "Basil"];

pub struct Environment {
    time: u32,
    all_names : Vec<String>,
    pub agents: Vec<Agent>,
}

impl Environment {
    pub fn create(ollama: Ollama, num_agents: usize) -> Self {
        let mut new_env = Environment {
            time: 0,
            all_names: Vec::new(),
            agents: Vec::with_capacity(num_agents),
        };

        let all_names: Vec<_> = (0..num_agents).map(|i| POTENTIAL_NAMES[i].to_owned()).collect();

        for i in 0..num_agents {
            // TODO
            let name = POTENTIAL_NAMES[i].to_string();

            new_env
                .agents
                .push(Agent::new_random(ollama.clone(), &all_names, name, i as u8));
        }
        new_env.all_names = all_names;
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
            dbg!(
                self.agents[i].food,
                self.agents[i].money,
                self.agents[i].age
            );

            match action.action {
                Action::Work => {
                    self.agents[i].give_money(5);
                }
                Action::MakeFood => {
                    let amount = action.args.amount.unwrap();
                    self.agents[i].give_food(amount);
                    self.agents[i].money -= amount;
                }
                Action::GiveMoney => {
                    let trading_with = action.args.who_to_interact_with.unwrap();

                    let other_id = self.get_id_from_name(&trading_with);

                    // todo tell the LLM they're an idiot
                    let other_id = other_id.unwrap();

                    self.agents[other_id].give_money(action.args.amount.unwrap());
                }
                Action::GiveFood => {
                    let other_id =
                        self.get_id_from_name(&action.args.who_to_interact_with.unwrap());

                    // todo tell the LLM they're an idiot
                    let other_id = other_id.unwrap();

                    self.agents[other_id].give_food(action.args.amount.unwrap());
                }
                Action::Converse => {
                    // TODO tell LLM they're an idiot
                    let index = self
                        .get_id_from_name(&action.args.who_to_interact_with.unwrap())
                        .unwrap();
                    let name = self.agents[i].name.clone();
                    let msg_back = self.agents[index]
                        .send_msg(action.args.message.unwrap(), &name)
                        .await;
                    self.agents[i].listen(msg_back, &name).await;
                },
                Action::Reproduce => {
                    let index = self
                        .get_id_from_name(&action.args.who_to_interact_with.unwrap())
                        .unwrap();
                    let name = self.agents[i].name.clone();
                    let accepted = self.agents[index]
                        .propose(action.args.message.unwrap(), &name)
                        .await?;
                    if accepted {
                        let honesty = self.agents[index].honesty;
                        let socialness = self.agents[index].socialness;
                        let selfishness = self.agents[index].selfishness;
                        let compassion = self.agents[index].compassion;
                        let new_agent = self.agents[i].reproduce(honesty, socialness, selfishness, compassion, &self.all_names);

                        let mut new_names = self.all_names.to_vec();
                        new_names.push(new_agent.name.clone());

                        self.all_names = new_names;
                    }
                }
                Action::Broadcast => {
                    let name = self.agents[i].name.clone();
                    for j in 0..self.agents.len() {
                        if i == j {
                            continue;
                        }
                        self.agents[j]
                            .listen(action.args.message.clone().unwrap(), &name)
                            .await;
                    }
                }
            }

            if self.agents[i].age() {
                let name = self.agents[i].name.clone();
                for j in 0..self.agents.len() {
                    if i == j {
                        continue;
                    }
                    self.agents[j].listen(format!("{} has died. Rest in peace.", &name), &name).await;
                }
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
