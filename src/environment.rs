use crate::{action::Action, agent::Agent};
use faker_rand::en_us::names::FirstName;
use ollama_rs::{generation::chat::ChatMessage, Ollama};
use rand::random;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{native_token::sol_to_lamports, signature::{keypair_from_seed, Keypair}, signer::{keypair::generate_seed_from_seed_phrase_and_passphrase, Signer}, system_program, system_transaction};

pub const SEED_PHRASE : &str = "aquariLLM";
pub const PASSPHRASE : &str = "simmed d";

// replace with keypair from config
pub const KEYPAIR_BYTES : &[u8] = &[211,101,102,180,52,150,64,170,128,3,135,231,115,148,235,185,159,220,105,1,81,193,179,146,81,229,142,228,199,136,99,171,80,143,31,28,107,255,223,95,171,254,205,10,90,45,116,163,158,54,133,106,63,181,154,77,58,152,44,18,105,78,24,66];

pub struct Environment {
    time: u32,
    all_names: Vec<String>,
    pub agents: Vec<Agent>,
    pub client: RpcClient,
    pub keypair: Keypair
}

impl Environment {
    pub async fn create(ollama: Ollama, client : RpcClient, num_agents: usize) -> Self {
        let seed = generate_seed_from_seed_phrase_and_passphrase(SEED_PHRASE, PASSPHRASE);
        let keypair = keypair_from_seed(&seed).unwrap();
        let ctx = client.get_latest_blockhash().unwrap();
        println!("Creating environment..");
        println!("Env keypair: {}", keypair.pubkey().to_string());

        if client.get_balance(&keypair.pubkey()).expect("Error getting account balance") == 0 {
            let tx = system_transaction::create_account(
                &Keypair::from_bytes(KEYPAIR_BYTES).unwrap(),
                &keypair,
                ctx,
                sol_to_lamports(10.0),
                0,
                &system_program::ID
            );
            let _cfm = client.send_and_confirm_transaction(&tx).expect("Error creating env account");
        }
        let _confirmation = client.request_airdrop(&keypair.pubkey(), sol_to_lamports(5.0)).expect("Error airdropping sol");

        let mut new_env = Environment {
            time: 0,
            all_names: Vec::new(),
            agents: Vec::with_capacity(num_agents),
            client,
            keypair
        };

        let mut all_names: Vec<_> = Vec::new();

        for _ in 0..num_agents {
            let name = random::<FirstName>().to_string();
            all_names.push(name);
        }

        println!("Creating agents..");
        for i in 0..num_agents {
            let name = all_names[i].clone();
            new_env
                .agents
                .push(Agent::new_random(ollama.clone(), &new_env.client, &new_env.keypair, &all_names, name));
        }
        new_env.all_names = all_names;
        new_env
    }

    pub async fn run_timestep(&mut self) -> anyhow::Result<()> {
        println!(
            "\n\n[INFO] There are {} people in the community",
            self.agents.len()
        );

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
                    if let Some(other_id) =
                        self.get_id_from_name(&action.args.who_to_interact_with.unwrap())
                    {
                        let name = self.agents[i].name.clone();
                        self.agents[other_id].give_money(action.args.amount.unwrap(), &name, &self.keypair, &self.client);
                    } else {
                        self.agents[i].history.push(
                            ChatMessage::system("You tried to interact with someone who is not in the community! Please interact with members of the community".to_string())
                            );
                    }
                }
                Action::GiveFood => {
                    if let Some(other_id) =
                        self.get_id_from_name(&action.args.who_to_interact_with.unwrap())
                    {
                        let name = self.agents[i].name.clone();
                        self.agents[other_id].give_food(action.args.amount.unwrap(), &name);
                    } else {
                        self.agents[i].history.push(
                            ChatMessage::system("You tried to interact with someone who is not in the community! Please interact with members of the community".to_string()
                            ));
                    }
                }
                Action::Converse => {
                    if let Some(other_id) =
                        self.get_id_from_name(&action.args.who_to_interact_with.unwrap())
                    {
                        let name = self.agents[i].name.clone();
                        let msg_back = self.agents[other_id]
                            .send_msg(action.args.message.unwrap(), &name)
                            .await;
                        self.agents[i].listen(msg_back, &name).await;
                    } else {
                        self.agents[i].history.push(
                            ChatMessage::system("You tried to interact with someone who is not in the community! Please interact with members of the community".to_string()
                            ));
                    }
                }
                Action::Reproduce => {
                    if let Some(index) =
                        self.get_id_from_name(&action.args.who_to_interact_with.unwrap())
                    {
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

                            println!("[DEBUG] New person: {}", new_agent.name);

                            let mut new_names = self.all_names.to_vec();
                            new_names.push(new_agent.name.clone());

                            for j in 0..self.agents.len() {
                                if i == j {
                                    continue;
                                }
                                self.agents[j]
                                    .listen(
                                        format!(
                                            "There's a new member of the community named {}!",
                                            new_agent.name.clone()
                                        ),
                                        &name,
                                    )
                                    .await;

                                self.agents[j]
                                    .listen(
                                        format!(
                                            "There's a new member of the community named {}!",
                                            new_agent.name.clone()
                                        ),
                                        &name,
                                    )
                                    .await;
                            }

                            self.agents.push(new_agent);
                            self.all_names = new_names;
                        }
                    } else {
                        self.agents[i].history.push(
                                ChatMessage::system("You tried to interact with someone who is not in the community! Please interact with members of the community".to_string()
                                ));
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

                println!("[DEBUG] {name} has died");

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
