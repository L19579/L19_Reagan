
pub mod swap_repo;
pub mod prelude;
pub mod prompt;
pub mod schema;
pub use { prelude::*, schema::*, prompt::* };

static mut FILE_BUFFER: [u8; 30_000] = [0; 30_000];

pub struct Scout{
    pub config: Config,
}

impl<'de> Scout{
    pub fn new() -> Result<Self>{
        let config = Self::load_config().unwrap();     
        return Ok(Self{ config });
    }

    fn load_config() -> Result<Config>{ // Issues here. Recursive calls for Config. Priority.
        println!("Starting program.");
        let rpc_client = RpcClient::new(RPC_SOLANA_LINK);
        let config_paths = match Self::process_toml::<ConfigPaths>(CONFIG_PATH){
            Ok(c) => c,
            Err(e) => {
                let error = format!("{} ; e: {}", Error::UnableToBuildConfig.to_string(), e); 
                show_statement(StatementType::BoldError, &error); 
                Error::UnableToBuildConfig.to_string(); 
                return Err(anyhow!("{}", e));
            } 
        };
        let user_config = Self::process_toml::<UserConfig>(&config_paths.user_config_path).unwrap().format(&rpc_client)?;
        let mut token_swap: Option<FormattedTokenSwap> = None;
        if config_paths.token_swap_config_path.is_some(){ 
            token_swap = Some(
                Self::process_toml::<TokenSwap>(&config_paths.token_swap_config_path.as_ref().unwrap())
                .unwrap().format(&rpc_client, &user_config)?);
        }
        let mut orca_token_swap: Option<FormattedOrcaTokenSwap> = None;
        if config_paths.orca_token_swap_config_path.is_some(){ 
            orca_token_swap = Some(
                Self::process_toml::<OrcaTokenSwap>(&config_paths.orca_token_swap_config_path.as_ref().unwrap())
                .unwrap().format(&rpc_client, &user_config)?);
        }
        let mut orca_token_swap_v2: Option<FormattedOrcaTokenSwapV2> = None;
        if config_paths.orca_token_swap_v2_config_path.is_some(){ 
            orca_token_swap_v2 = Some(
                Self::process_toml::<OrcaTokenSwapV2>(&config_paths.orca_token_swap_v2_config_path.unwrap())
                .unwrap().format(&rpc_client, &user_config)?);
        }

        return Ok(Config{
            user_config: user_config,
            token_swap: token_swap,
            orca_token_swap: orca_token_swap,
            orca_token_swap_v2: orca_token_swap_v2,
        });
    }
    
    pub fn capture_local(user_config: &FormattedUserConfig, swap_input: &[(&TokenSwapPoolInfo, u64, u64)]) -> Result<()>{
        let rpc_client = RpcClient::new(RPC_SSO_LINK);
        
        if swap_input.len() > 3 || swap_input.len() < 2{
            let error = Error::LegCountDisallowed.to_string();
            show_statement(StatementType::Error, &error);
            return Err(anyhow!(error));
        }
        let swap_data = swap_input.iter().map(|s| swap_repo::instruction::SwapInstruction::Swap(
            swap_repo::instruction::Swap{amount_in: s.1, minimum_amount_out: s.2,
        }).pack()).collect::<Vec<Vec<u8>>>(); 
        let instructions = swap_input.iter().enumerate().map(|(i, s)| 
            Instruction::new_with_bytes(s.0.pool.swap_program, &swap_data[i], s.0.pool.account_metas.clone())
        ).collect::<Vec<Instruction>>();
        let message = Message::new(&instructions, Some(&user_config.signer_key));
        let blockhash = rpc_client.get_latest_blockhash().unwrap();
        let signed_tx = Transaction::new(&[&user_config.signer_keypair], message, blockhash);
        //let tx_sig = rpc_client.send_and_confirm_transaction(&signed_tx).unwrap();
        let commitment_config = CommitmentConfig::confirmed();
        let transaction_config = RpcSendTransactionConfig{
            skip_preflight: true, preflight_commitment: None,
            encoding: None, max_retries: None, min_context_slot: None,
        };
        let tx_sig = rpc_client.send_and_confirm_transaction_with_spinner_and_config(
                &signed_tx, commitment_config, transaction_config).unwrap();
        println!("tx_sig: {}", tx_sig);

        return Ok(());
    }
    
    pub fn capture_local_via_program(user_config: &FormattedUserConfig, swap_input: &[(&TokenSwapPoolInfo, u64, u64)]) 
    -> Result<()>{
        let rpc_client = RpcClient::new(RPC_SSO_LINK);
        if swap_input.len() > 3 || swap_input.len() < 2{
            let error = Error::LegCountDisallowed.to_string();
            show_statement(StatementType::Error, &error);
            return Err(anyhow!(error));
        }
        let mut swap_data = swap_input.iter().map(|s| swap_repo::instruction::SwapInstruction::Swap(
            swap_repo::instruction::Swap{amount_in: s.1, minimum_amount_out: s.2,
        }).pack()).collect::<Vec<Vec<u8>>>(); 
        
        let mut account_metas = Vec::<AccountMeta>::new();
        let mut concatenated_data = Vec::<u8>::new();
        match swap_input.len(){
            2 => {
                swap_data.iter().map(|s_d|{concatenated_data.extend_from_slice(&s_d)}).collect::<Vec<_>>();
                concatenated_data.push(0);
                
                let amm_account_metas: Vec<AccountMeta> = swap_input.iter().map(|s_i|{
                    AccountMeta{
                        pubkey: s_i.0.pool.swap_program,
                        is_signer: false,
                        is_writable: false,
                    }
                }).collect();

                account_metas.push(swap_input[0].0.pool.account_metas[2].clone()); 
                account_metas.push(swap_input[0].0.pool.account_metas[9].clone()); 
                account_metas.push(amm_account_metas[0].clone());                  
                account_metas.push(amm_account_metas[1].clone());                  
                account_metas.push(swap_input[0].0.pool.account_metas[3].clone()); 
                account_metas.push(swap_input[1].0.pool.account_metas[3].clone()); 

                for s_i in swap_input{
                    _ = s_i.0.pool.account_metas.iter().enumerate().map(|(i, a_m)|{
                        match i{
                            0 | 1 | 4 | 5 | 7 | 8 => {
                                account_metas.push(a_m.clone())            
                            }
                            _ => (),
                        };
                    }).collect::<Vec<_>>();
                };
            },
            3 => {
                swap_data.iter().map(|s_d|{concatenated_data.extend_from_slice(&s_d)}).collect::<Vec<_>>();
                concatenated_data.push(1);

                let amm_account_metas: Vec<AccountMeta> = swap_input.iter().map(|s_i|{
                    AccountMeta{
                        pubkey: s_i.0.pool.swap_program,
                        is_signer: false,
                        is_writable: false,
                    }
                }).collect();
                
                account_metas.push(swap_input[0].0.pool.account_metas[2].clone()); 
                account_metas.push(swap_input[0].0.pool.account_metas[9].clone()); 
                account_metas.push(amm_account_metas[0].clone());                  
                account_metas.push(amm_account_metas[1].clone());                  
                account_metas.push(amm_account_metas[2].clone());                  
                account_metas.push(swap_input[0].0.pool.account_metas[3].clone()); 
                account_metas.push(swap_input[1].0.pool.account_metas[3].clone()); 
                account_metas.push(swap_input[2].0.pool.account_metas[3].clone()); 

                for s_i in swap_input{
                    _ = s_i.0.pool.account_metas.iter().enumerate().map(|(i, a_m)|{
                        match i{
                            0 | 1 | 4 | 5 | 7 | 8 => {
                                account_metas.push(a_m.clone())            
                            }
                            _ => (),
                        };
                    }).collect::<Vec<_>>();
                };
            },
            _ => ()
        }
        let l19_program = b58_to_pubkey(L19_PROGRAM_ID); 
        let instruction = Instruction::new_with_bytes(l19_program, &concatenated_data, account_metas);
        let message = Message::new(&[instruction], Some(&user_config.signer_key));
        let blockhash = rpc_client.get_latest_blockhash()?; 
        let signed_tx = Transaction::new(&[&user_config.signer_keypair], message, blockhash);
        let transaction_config = RpcSendTransactionConfig{
            skip_preflight: true, preflight_commitment: None,
            encoding: None, max_retries: None, min_context_slot: None,
        };
        let tx_sig = rpc_client.send_transaction_with_config(
                &signed_tx, transaction_config)?;
        println!("tx_sig: {}", tx_sig);
        
        return Ok(()); 
    }
    
    fn process_toml<T: serde::de::Deserialize<'de>>(path_s: &str) -> Result<T>{
        unsafe{
            let config_file = OpenOptions::new().read(true).open(path_s).unwrap();
            let file_len = config_file.metadata().unwrap().len() as usize;
            FILE_BUFFER = [0; 30_000];
            let mut buf_reader = BufReader::new(config_file);
            buf_reader.read(&mut FILE_BUFFER).unwrap();
            return Ok(toml::de::from_slice::<T>(&FILE_BUFFER[..file_len]).unwrap());
        }
    }
    
    /// Generates routes out of swap accounts in /data
    fn generate_routes_2(&self) -> Result<(Vec<(TokenSwapPoolInfo, Sender<TokenSwapPoolInfo>, 
    Receiver<TokenSwapPoolInfo>)>, Vec<Box<dyn ArbPathTrait>>)>{
        //extracting pools.
        let mut swap_programs = Vec::<Box<dyn NativeSwap>>::new();
        let mut arb_paths = Vec::<Box<dyn ArbPathTrait>>::new();
        if self.config.token_swap.is_some(){swap_programs.push(
            Box::new(self.config.token_swap.clone().unwrap()),
        )};
        if self.config.orca_token_swap.is_some(){swap_programs.push(
            Box::new(self.config.orca_token_swap.clone().unwrap()),
        )};
        if self.config.orca_token_swap_v2.is_some(){swap_programs.push(
            Box::new(self.config.orca_token_swap_v2.clone().unwrap()),
        )};
        let mut grouped_pools_senders_and_receivers = Vec::<(TokenSwapPoolInfo, 
            Sender<TokenSwapPoolInfo>, Receiver<TokenSwapPoolInfo>)>::new(); 
        swap_programs.iter().map(|s_p|{
            for pool in s_p.pools(){
                let mut token_p_info = [
                    TokenSwapPoolInfo::new(&pool), 
                    TokenSwapPoolInfo::new(&pool).reverse_source_destination(),
                ];
                token_p_info.iter().map(|t_p_i|{
                    let (sender, receiver) = bounded::<TokenSwapPoolInfo>(70);
                    grouped_pools_senders_and_receivers.push(
                        (t_p_i.clone(), sender, receiver)
                    ); 
                }).collect::<()>();
            } 
        }).collect::<()>();

        /*
        let approved_settlement_accounts = [ // user can restrict settlement token accounts by listing them here.  <-- RECOMMENDED. 
            "",                              // code launches every possible non-conflicting route otherwise.
        ];
        */

        //creatig paths
        for (i_a, p_a) in grouped_pools_senders_and_receivers.iter().enumerate(){
            let mut  found_settlement_match = false;
            approved_settlement_accounts.into_iter().for_each(|a_s_a|{
                if p_a.0.pool().account_keys[3].to_string() == a_s_a{
                    found_settlement_match = true;
                }
            });
            if found_settlement_match == false { continue };
            for (i_b, p_b) in grouped_pools_senders_and_receivers.iter().enumerate(){
                if i_a == i_b { continue; }
                    //ensuring consecutive swaps are not between the same pools for 2 legs
                    if p_a.0.pool().account_keys[0].to_string() == p_b.0.pool().account_keys[0].to_string(){
                        continue;
                    }
                if is_linked(&[&p_a.0, &p_b.0]){
                    arb_paths.push(Box::new(ArbPath::new(
                        vec![
                            p_a.0.clone(),
                            p_b.0.clone(),
                        ],
                        vec![
                            grouped_pools_senders_and_receivers[i_a].2.clone(), 
                            grouped_pools_senders_and_receivers[i_b].2.clone(), 
                        ]         
                    ).unwrap()));
                }
                for (i_c, p_c) in grouped_pools_senders_and_receivers.iter().enumerate(){
                    if i_b == i_c { continue; }
                    //ensuring consecutive swaps are not between the same pools for 3 legs
                    if p_b.0.pool().account_keys[0].to_string() == p_c.0.pool().account_keys[0].to_string(){
                        continue;
                    }
                    if is_linked(&[&p_a.0, &p_b.0, &p_c.0]){
                        arb_paths.push(Box::new(ArbPath::new(
                            vec![
                                p_a.0.clone(),
                                p_b.0.clone(),
                                p_c.0.clone(),
                            ],
                            vec![
                                grouped_pools_senders_and_receivers[i_a].2.clone(), 
                                grouped_pools_senders_and_receivers[i_b].2.clone(), 
                                grouped_pools_senders_and_receivers[i_c].2.clone(), 
                            ]         
                        ).unwrap()));
                    }
                }
            }
        }
        let unique_pools_and_senders = grouped_pools_senders_and_receivers.iter().map(|p_s_r|{
            (p_s_r.0.clone(), p_s_r.1.clone(), p_s_r.2.clone())
        }).collect::<Vec<(TokenSwapPoolInfo, Sender<TokenSwapPoolInfo>, Receiver<TokenSwapPoolInfo>)>>();
        
        thread::sleep(Duration::from_secs(1));
        let stmt = format!("{} routes loaded through {} markets", arb_paths.len(), unique_pools_and_senders.len());
        show_statement(StatementType::Success, &stmt);
        thread::sleep(Duration::from_secs(5));
        return Ok((unique_pools_and_senders, arb_paths));
    }

    pub fn launch_searchers(&self) -> Result<()>{
        let (mut target_pool_infos_and_senders, arb_paths) = self.generate_routes_2()?;
        let capture_lock = Arc::new(RwLock::new(false)); // This value never changes. Lock is needed to prevent 3+ executions in the same block.
        let mut fetch_handles = vec![];
        {
            fetch_handles = target_pool_infos_and_senders.into_iter().map(|t_p_i_original|{ //temp: .._original
                thread::spawn(move||{
                    let mut relaunch_attempt_count = 0u8;
                    '_recovery_: loop{
                        let mut t_p_i = t_p_i_original.clone();
                        '_operational_loop: loop{
                            thread::sleep(Duration::from_millis(40));
                            if t_p_i.2.len() > 2{
                                for _ in &t_p_i.2{ // consume all data in channel
                                    continue;
                                }                  
                            } 

                            let rpc_client = RpcClient::new(RPC_SSO_LINK);
                            match t_p_i.0.update_vault_quantities(&rpc_client){
                                Err(_) => {
                                    thread::sleep(Duration::from_millis(100));
                                    continue;
                                },
                                _ => (), 
                            };
                            (0..100).into_iter().for_each(|_|{
                                match t_p_i.1.try_send(t_p_i.0.clone()){
                                    _ => (),
                                }
                            });
                            thread::sleep(Duration::from_millis(200));
                            (0..100).into_iter().for_each(|_|{
                                match t_p_i.2.try_recv(){
                                    _ => (),
                                }
                            });
                    }
                })    
            }).collect();
        }
        thread::sleep(Duration::from_secs(1)); // allowing pool_infos to fetch price
        let scout_handles = arb_paths.iter().map(|a| a.observe(self.config.user_config.gen_copy(), 
            Arc::clone(&capture_lock)).unwrap()).collect::<Vec<thread::JoinHandle<()>>>();
        _ = scout_handles.into_iter().map(|h| h.join().unwrap()).collect::<Vec<_>>();
        _ = fetch_handles.into_iter().map(|h| h.join().unwrap()).collect::<Vec<_>>();
        return Ok(());
    }
}

pub fn is_linked<T: PoolInfo + Clone>(pool_infos: &[&T]) -> bool{
    let mut result = true;
    let max_index = pool_infos.len() - 1;
    _ = pool_infos.iter().enumerate().map(|(i, _)|{
        if i >= max_index { 
            return ();
        }; 
        if i == 0 {
                if pool_infos[max_index].user_source_and_destination_token_accounts().1.1
                != pool_infos[i].user_source_and_destination_token_accounts().0.1{
                    result = false;
                    //println!("trace --- 1st non matching\n\t{}\n\t{}", pool_infos[max_index].user_source_and_destination_token_accounts().1.1,
                    //pool_infos[i].user_source_and_destination_token_accounts().0.1);
            return ();
                }
                //println!("trace --- 1st matching \n\t{}\n\t{}", pool_infos[max_index].user_source_and_destination_token_accounts().1.1,
                //pool_infos[i].user_source_and_destination_token_accounts().0.1);
        };
        match i {
            _ => {
                if pool_infos[i].user_source_and_destination_token_accounts().1.1
                != pool_infos[i+1].user_source_and_destination_token_accounts().0.1{
                    result = false;
                    //println!("trace --- 2nd non matching\n\t{}\n\t{}", pool_infos[max_index].user_source_and_destination_token_accounts().1.1,
                    //pool_infos[i].user_source_and_destination_token_accounts().0.1);
            return ();
                }
                //println!("trace --- 2nd matching accounts\n\t{}\n\t{}", pool_infos[max_index].user_source_and_destination_token_accounts().1.1,
                //pool_infos[i].user_source_and_destination_token_accounts().0.1);
            },
        }
    }).collect::<Vec<_>>();         
    return result;
}


pub fn b58_to_pubkey(b58: &str) -> Pubkey{
    return Pubkey::new(&bs58::decode(b58).into_vec().unwrap()); 
}

pub fn corresponding_user_wallet_mint(rpc_client: &RpcClient, 
user_token_accounts: &[FormattedTokenAccount], vault_token_account: Pubkey) -> Result<Pubkey>{
    let spl_mint_key = spl_mint_key(&rpc_client, &vault_token_account).unwrap();
    for user_token_account in user_token_accounts{
        println!("TRACE --- {}", format!("{} / {}", user_token_account.mint, spl_mint_key));
        if user_token_account.mint == spl_mint_key{
            println!("TRACE --- Returned");
            return Ok(user_token_account.key);
        } 
    }
    return Err(anyhow!(Error::WalletVaultMintKeyMismatch.to_string()));
}

pub fn spl_mint_key<T: Display>(rpc_client: &RpcClient, spl: &T) -> Result<Pubkey>{
    let spl_key = b58_to_pubkey(&format!("{}", spl));
    let ui_token_account = rpc_client.get_token_account(&spl_key)?;
    if ui_token_account.is_none(){
        let error = Error::UnableToFindMint.to_string();
        show_statement(StatementType::Error, &error);
        return Err(anyhow!(error));
    };
    return Ok(b58_to_pubkey(&ui_token_account.unwrap().mint));
}

pub trait PoolInfo{
    fn swap_program(&self) -> Pubkey;
    fn pool(&self) -> &FormattedPool; // Needs review.
    fn user_starting_balance(&self) -> &Option<u64>;
    fn last_updated(&self) -> &Option<DateTime<Utc>>;
    fn reverse_source_destination(&self) -> Self;
    fn update_and_calculate_out_amount(&mut self, rpc_client: &RpcClient, in_amount: u64) -> Result<u64>;
    fn update_vault_quantities(&mut self, rpc_client: &RpcClient) -> Result<()>;
    fn calculate_out_amount(&self, /*rpc_client: &RpcClient,*/ in_amount: u64) -> Result<u64>;
    fn user_source_and_destination_token_accounts(&self) -> ((String, &Pubkey), (String, &Pubkey));
    fn pool_source_and_destination_token_accounts(&self) -> ((String, &Pubkey), (String, &Pubkey));
}

#[derive(Clone, PartialEq, Debug)]
pub struct TokenSwapPoolInfo{
    pub pool: FormattedPool,
    pub user_starting_balance: Option<u64>,
    pub source_quantity: UiTokenAmount,
    pub destination_quantity: UiTokenAmount,
    pub last_updated: Option<DateTime<Utc>>,
}
impl TokenSwapPoolInfo{
    pub fn new(pool: &FormattedPool) -> TokenSwapPoolInfo{
        let last_updated = None; 
        
        let null_ui_token_amount = UiTokenAmount{
            ui_amount: Some(0f64),
            decimals: 1,
            amount: "0".to_string(),
            ui_amount_string: "1".to_string(),
        };

        return TokenSwapPoolInfo{
            pool: pool.clone(),
            user_starting_balance: None,
            source_quantity: null_ui_token_amount.clone(),
            destination_quantity: null_ui_token_amount,
            last_updated,
        };
    }
}

impl PoolInfo for TokenSwapPoolInfo{
    fn swap_program(&self) -> Pubkey{
        self.pool.swap_program
    }
    
    fn reverse_source_destination(&self) -> TokenSwapPoolInfo{
        let mut new_keys = self.pool.account_keys.clone(); 
        let mut new_metas = self.pool.account_metas.clone();
         
        let reverse_pattern: [(usize, usize); 4] = [(4,5), (5,4), (3,6), (6,3)];
        for (i, j) in reverse_pattern{
            new_keys[i] = self.pool.account_keys[j].clone(); 
            new_metas[i] = self.pool.account_metas[j].clone();
        }

        let pool = FormattedPool{ 
            in_symbol: self.pool.out_symbol.clone(),
            out_symbol: self.pool.in_symbol.clone(),
            swap_program: self.pool.swap_program.clone(),
            account_keys: new_keys, 
            account_metas: new_metas 
        };
        
        return Self::new(&pool);
    }
    
    fn pool(&self) -> &FormattedPool{
        return &self.pool;
    }

    fn user_starting_balance(&self) -> &Option<u64>{
        return &self.user_starting_balance; 
    }
    fn last_updated(&self) -> &Option<DateTime<Utc>>{
        return &self.last_updated;
    }

    fn update_and_calculate_out_amount(&mut self, rpc_client: &RpcClient, in_amount: u64) -> Result<u64>{
        if  self.last_updated.is_none() 
            || Utc::now().signed_duration_since(self.last_updated.unwrap()).num_seconds() > 2{
            self.update_vault_quantities(rpc_client)?;
        }
        return self.calculate_out_amount(in_amount);
    }

    fn update_vault_quantities(&mut self, rpc_client: &RpcClient) -> Result<()>{
        let user_starting_account = &self.pool.account_keys[3]; 
        self.user_starting_balance = Some(match rpc_client.get_token_account_balance(user_starting_account){
            Ok(ui) => {
                self.last_updated = Some(Utc::now());
                (ui.ui_amount.unwrap() * 10f64.powi(ui.decimals as i32)) as u64 // --- Not sure when this would == None. Mod if common.
            },
            Err(_) => {
                let error = Error::UnableToFetchTokenAccountBalance(user_starting_account).to_string();
                show_statement(StatementType::Warning, &error);  
                return Err(anyhow!(error));
            }
        });
        self.source_quantity = rpc_client.get_token_account_balance(&self.pool.account_keys[4])?; 
        self.destination_quantity = rpc_client.get_token_account_balance(&self.pool.account_keys[5])?; 
        return Ok(()); 
    }
        
    /// Pools selected use a constant product curve: (a + a_in) * (b + b_out) = K
    fn calculate_out_amount(&self, /*rpc_client: &RpcClient,*/ in_amount: u64) -> Result<u64>{
        if  self.last_updated.is_none(){
            println!("TRACE --- last updated is none");
            return Err(anyhow!(Error::PoolQuantitiesNotUpdated.to_string()));
        }
        if self.source_quantity.ui_amount.is_none() || self.destination_quantity.ui_amount.is_none(){
            let error = Error::VaultBalanceZero.to_string(); 
            show_statement(StatementType::Warning, &error); 
            return Err(anyhow!("{}", error));
            
        }
       /* 
        println!("TRACE --- source_quantity: {}", self.source_quantity.ui_amount.unwrap() 
            * 10f64.powi(self.source_quantity.decimals as i32));
        println!("TRACE --- destination_quantity: {}", self.destination_quantity.ui_amount.unwrap() 
            * 10f64.powi(self.destination_quantity.decimals as i32));
        */
        let source_quantity = (self.source_quantity.ui_amount.unwrap() 
            * 10f64.powi(self.source_quantity.decimals as i32)) as i128;
        let destination_quantity = (self.destination_quantity.ui_amount.unwrap() 
            * 10f64.powi(self.destination_quantity.decimals as i32)) as i128;
        let out_amount_pre_fees = (destination_quantity - ((source_quantity * destination_quantity)
            /(source_quantity + in_amount as i128))) as f64; 
        
        return Ok( (out_amount_pre_fees * (1f64 - MAX_FEE)) as u64)
    }
    
    fn user_source_and_destination_token_accounts(&self) -> ((String, &Pubkey), (String, &Pubkey)){ // This may not apply to some pools.
        let keys = &self.pool.account_keys;
        return ((self.pool.in_symbol.clone(), &keys[3]), (self.pool.out_symbol.clone(), &keys[6]));
    } 

    fn pool_source_and_destination_token_accounts(&self) -> ((String, &Pubkey), (String, &Pubkey)){ // This may not apply to some pools.
        let keys = &self.pool.account_keys;
        return ((self.pool.in_symbol.clone(), &keys[4]), (self.pool.out_symbol.clone(), &keys[5]));
    } 
}

//Needed for generate_routes_2() 
pub trait ArbPathTrait{
    // Not observe() a req but obv include it.
    fn observe(&self, user_config: FormattedUserConfig, capture_lock: Arc<RwLock<bool>>) 
        -> std::io::Result<thread::JoinHandle::<()>>;
}

#[derive(Clone)]
pub struct ArbPath<T: PoolInfo + Send + Sync>{
    //pub pool_infos: Vec<Arc<RwLock<T>>>,
    pub pool_infos_receiver: Vec<Receiver<T>>,
    pub route_description: String,
    silencer: Option<T>, //temp
}

impl<T: PoolInfo + 'static + Send + Sync> ArbPath<T>{ // study static impl with spawn.
    pub fn new(proposed_pool_infos_model: Vec<T>, receiver: Vec<Receiver<T>>) 
    -> Result<Self>{
        let max_index = proposed_pool_infos_model.len() - 1;
        let pool_infos = proposed_pool_infos_model;//.iter().map(|p| p.clon).collect::<Vec<T>>();
        let settlement_symbol = pool_infos[0].pool_source_and_destination_token_accounts().0.0.clone();
        let mut route_description = String::new();
        _ = pool_infos.iter().enumerate().map(|(i, p)| { 
            if i == 0 {
                route_description.push_str(&format!("{}/{} ({})", 
                    pool_infos[i].pool_source_and_destination_token_accounts().0.0.clone(),
                    pool_infos[i].pool_source_and_destination_token_accounts().1.0.clone(),
                    &pool_infos[i].swap_program().to_string()[..3],
                ));    
                return ();
            }
            route_description.push_str(&format!(" > {}/{} ({})", 
                p.pool_source_and_destination_token_accounts().0.0.clone(),
                p.pool_source_and_destination_token_accounts().1.0.clone(),
                &p.swap_program().to_string()[..3],
            ));
        }).collect::<Vec<_>>();
        show_statement(StatementType::Success, &format!("{} : {} successfully loaded", route_description, max_index + 1));
        return Ok(Self{ pool_infos_receiver: receiver, route_description,silencer: None }); 
    }
}
/// This commits half of an accounts available balance.
impl ArbPathTrait for ArbPath<TokenSwapPoolInfo>{
    fn observe(&self, user_config: FormattedUserConfig, capture_lock: Arc<RwLock<bool>>) 
    -> std::io::Result<thread::JoinHandle::<()>>{
        let pool_infos_receiver = self.pool_infos_receiver.clone();
        let builder = thread::Builder::new().name(self.route_description.clone());
        return builder.spawn(move ||{
        let thread_name = String::from(thread::current().name().unwrap());
        println!("Thread \"{}\" is online", thread_name);
            let max_slip_per_swap = 1f64 - (MAX_SLIPPAGE / (pool_infos_receiver.len()) as f64);  // --------------
            let mut relaunch_attempt_count = 0u8;
            'recovery_loop: loop{
            match catch_unwind(||{
                'operational_loop: loop{
                    thread::sleep(Duration::from_millis(rand::thread_rng().gen_range(70..200)));
                        let mut pool_infos = Vec::<>::new();
                        for p_recv in &pool_infos_receiver{
                            match p_recv.recv_timeout(Duration::from_millis(100)){
                                Ok(p) => pool_infos.push(p),
                                _ => continue 'operational_loop
                            };
                        }
                        println!("TRACE --- RECVD ALL");
                        
                        let mut is_updated = [true; 10];

                        _ = pool_infos.iter().enumerate().map(|(i, p)| {
                            let duration_since_last_updated = Utc::now().signed_duration_since(p.last_updated().unwrap())
                                .num_milliseconds();
                            is_updated[i] = p.last_updated().is_some() &&
                            duration_since_last_updated < 900; // Hard to stay under 1 sec.
                            let stmt = format!("duration since last_updated: ({})ms",
                                duration_since_last_updated);
                            show_statement(StatementType::Test, &stmt);
                        }).collect::<Vec<_>>();
                        if is_updated != [true; 10]{ 
                            let stmt = format!("{:<19} : one or more pools not updated rechecking.", 
                                thread::current().name().unwrap_or("unamed thread"));
                            show_statement(StatementType::Warning, &stmt);
                            continue;
                        };
                        let user_starting_balance = pool_infos[0].user_starting_balance().as_ref();
                        if user_starting_balance.is_none() || *user_starting_balance.unwrap() < 10 { 
                            let stmt = format!("{:<19} : user token in account not updated or near empty, rechecking.", 
                                thread::current().name().unwrap_or("unamed thread"));
                            show_statement(StatementType::Warning, &stmt);
                            if user_starting_balance.is_none(){
                                println!("\tTRACE --- token account balance not updated. Rechecking");
                                continue;
                            };
                            if user_starting_balance.is_some(){
                                println!("\tTRACE --- token account balance is updated w/ a zero balance. Ending thread");
                                return ();
                            };
                        }

                        let mut in_out_amounts = Vec::<(u64, u64)>::new();
                        let user_starting_balance = user_starting_balance.unwrap() / 2; // --------------------------------------- Add opt to kill after x loops
                        in_out_amounts.push((user_starting_balance, (pool_infos[0]
                            .calculate_out_amount(user_starting_balance).unwrap() as f64 * max_slip_per_swap) as u64));  
                        _ = pool_infos.iter().enumerate().map(|(i, p)|{
                            if i != 0 {
                                let last_out = in_out_amounts.last().unwrap().1;
                                in_out_amounts.push((last_out, (p.calculate_out_amount(last_out).unwrap() as f64 * max_slip_per_swap) as u64));
                            };
                        }).collect::<Vec<_>>();    
                        
                        let pnl = in_out_amounts.last().unwrap().1 as i64 - in_out_amounts[0].0 as i64;
                        let target_increase = in_out_amounts[0].0 as f64 * 0.004f64; // profit capture target is 0.4%
                        if (in_out_amounts.last().unwrap().1 as f64 - in_out_amounts[0].0 as f64) >= target_increase {
                            match capture_lock.try_read(){  // prevents cross capture attempt in the same block.
                                Ok(_) => thread::sleep(Duration::from_millis(10)),
                                _ => continue 'operational_loop,
                            };
                            let settlement_symbol = pool_infos[0].pool_source_and_destination_token_accounts().0.0;
                            let stmt = format!("route: {} - starting balance {} {} ; ending balance {} {} ; pnl {} {}", 
                                thread_name, in_out_amounts[0].0, settlement_symbol, 
                                in_out_amounts.last().unwrap().1, settlement_symbol, pnl, settlement_symbol);
                            show_statement(StatementType::OpportunityFound, &stmt);
                            println!("TRACE --- target_increase {} ; actual: {}", target_increase as u64, pnl);
                            
                            //println!("TRACE ---- NATS TRIGGER CALLED!");

                            let mut swap_input = pool_infos.iter().enumerate().map(|(i, p)| (p, in_out_amounts[i].0, 0))
                                .collect::<Vec<(&TokenSwapPoolInfo, u64, u64)>>();
                            for i in 0..swap_input.len(){
                                swap_input[i].2 = in_out_amounts[i].1;
                            }
                            //Scout::capture_local(&user_config, &swap_input).unwrap();
                            // ^ user can use fn above to capture w/o on chain program. Slippage
                            // vulnerability is higher w/o cpi.
                            match Scout::capture_local_via_program(&user_config, &swap_input){
                                Err(_) => {
                                    show_statement(StatementType::Error, "Error on capture attempt via program. continuing program");
                                    continue;
                                }
                                _ => (),
                            };
                            thread::sleep(Duration::from_secs(30));

                            continue 'operational_loop;
                        }
                        let settlement_symbol = pool_infos[0].pool_source_and_destination_token_accounts().0.0;
                        
                        let stmt = format!("{} --- route: {} - starting balance {} {} ; ending balance {} {} ; pnl {} {}", 
                            pool_infos[0].last_updated().unwrap(), thread_name, 
                            in_out_amounts[0].0, settlement_symbol, in_out_amounts.last().unwrap().1, 
                            settlement_symbol, pnl, settlement_symbol);
                        println!("TRACE --- No oppotunity found: {}", stmt);
                }
            }){
                Err(_) => {
                    if relaunch_attempt_count > 3 { 
                        let stmt = format!("Failed to relaunch thread {} times. Cancelling job.", 
                        relaunch_attempt_count);
                        show_statement(StatementType::Error, &stmt);
                        break;
                    }
                    relaunch_attempt_count += 1;
                    let stmt = format!("ArbPath thread crashed, error code ignored, relaunching thread.");
                    show_statement(StatementType::Error, &stmt);
                    continue
                },
                _ => break 'recovery_loop,
            }}
        });
    }

    /// NATS implementation
    pub fn mock_nats_trigger(){
        // REMOVED. Up to user to create distributed compute logic.  
    }
}
