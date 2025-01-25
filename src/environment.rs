use crate::{action::Action, agent::Agent};
use faker_rand::en_us::names::FirstName;
use ollama_rs::Ollama;
use rand::random;

pub struct Environment {
    time: u32,
    all_names: Vec<String>,
    pub agents: Vec<Agent>,
}

impl Environment {
    pub fn create(ollama: Ollama, num_agents: usize) -> Self {
        let mut new_env = Environment {
            time: 0,
            all_names: Vec::new(),
            agents: Vec::with_capacity(num_agents),
        };

        let mut all_names: Vec<_> = Vec::new();
      
        for _ in 0..num_agents {
            let name = random::<FirstName>().to_string();
            all_names.push(name);
        }

        for i in 0..num_agents {

            let name = all_names[i].clone();
            new_env
                .agents
                .push(Agent::new_random(ollama.clone(), &all_names, name));
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
            println!();
            println!(
                "[DEBUG] {}: (thinking) {}",
                self.agents[i].name, action.thinking
            );
            println!(
                "[DEBUG] {}: took action {:?} with params {:?}",
                self.agents[i].name, action.action, action.args
            );
            println!(
                "[DEBUG] {}: {} food, {} money, {} age",
                self.agents[i].name, self.agents[i].food, self.agents[i].money, self.agents[i].age
            );

            match action.action {
                Action::Work => {
                    self.agents[i].work();
                }
                Action::MakeFood => {
                    self.agents[i].make_food();
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
                }
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
                        let food_ability = self.agents[index].food_ability;
                        let new_agent = self.agents[i].reproduce(
                            honesty,
                            socialness,
                            selfishness,
                            compassion,
                            food_ability,
                            &self.all_names,
                        );

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
                    self.agents[j]
                        .listen(format!("{} has died. Rest in peace.", &name), &name)
                        .await;
                }
                dead.push(i);
            }
        }

        for d in dead.into_iter().rev() {
            self.agents.remove(d);
        }
        self.time += 1;
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
