use crate::agent::Agent;

pub struct Environment {
    time : u32,
    agents : Vec<Agent>,
}

impl Environment {
    pub fn create(num_agents : usize) -> Self {
        let mut new_env = Environment{time: 0, agents: Vec::with_capacity(num_agents)};
        for i in 0..num_agents {
            new_env.agents.push(Agent::new_random(i as u8));
        }
        new_env
    }
    pub fn run_timestep (&mut self) {
        let mut actions = Vec::with_capacity(self.agents.len());
        for agent in self.agents.iter_mut() {
            actions.push(agent.step());
        }

        for (i, action) in actions.into_iter().enumerate() {
            match action {
                crate::action::Action::MakeFood => todo!(),
                crate::action::Action::GiveMoney(give_info) => {
                    self.agents[give_info.agent_trading_with].give_money(give_info.amount);
                },
                crate::action::Action::GiveFood(give_info) => {
                    self.agents[give_info.agent_trading_with].give_food(give_info.amount);
                },
                crate::action::Action::Converse(msg, index) => {
                    let name = self.agents[i].name.clone();
                    let msg_back = self.agents[index].send_msg(msg, &name);
                    self.agents[i].listen(msg_back, &name);
                },
                crate::action::Action::Broadcast(msg, index) => {
                    let name = self.agents[index].name.clone();
                    for i in 0..self.agents.len() {
                        if i == index {
                            continue;
                        }
                        self.agents[i].listen(msg.clone(), &name);
                    }
                },
            }
        }
    }
}
