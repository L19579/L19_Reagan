//Lots of pointless steps w/ format() fns. User can simplify.

use crate::{
    RpcClient, Pubkey, AccountMeta, 
    serde::*, prelude::TOKEN_PROGRAM_ID,
    keypair, Keypair,
    Path, Result,
    b58_to_pubkey,
    TokenAccountsFilter,
};

pub trait NativeSwap{
    fn pools(&self) -> Vec<FormattedPool>;
}
impl NativeSwap for FormattedTokenSwap{
    fn pools(&self) -> Vec<FormattedPool>{
        return self.pools.clone();
    }
}
impl NativeSwap for FormattedOrcaTokenSwap{
    fn pools(&self) -> Vec<FormattedPool>{
        return self.pools.clone();
    }
}
impl NativeSwap for FormattedOrcaTokenSwapV2{
    fn pools(&self) -> Vec<FormattedPool>{
        return self.pools.clone();
    }
}

#[derive(Debug)]
pub struct Config{ // Preferable to have FormattedTokenSwap in a single arr
    pub user_config: FormattedUserConfig,
    pub token_swap: Option<FormattedTokenSwap>,
    pub orca_token_swap: Option<FormattedOrcaTokenSwap>,
    pub orca_token_swap_v2: Option<FormattedOrcaTokenSwapV2>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ConfigPaths{
    pub user_config_path: String,
    pub token_swap_config_path: Option<String>,
    pub orca_token_swap_config_path: Option<String>,
    pub orca_token_swap_v2_config_path: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct UserConfig{
    pub signer_key: String,
    pub signer_keypair_path: String,
    pub token_accounts: Vec<TokenAccount>, // Drop this as a requirement.
}
impl UserConfig{
    pub fn deprecated_format(&self, rpc_client: &RpcClient) -> Result<FormattedUserConfig>{
        return Ok(FormattedUserConfig{
            signer_key: b58_to_pubkey(&self.signer_key),
            signer_keypair:  
                keypair::read_keypair_file(Path::new(&self.signer_keypair_path.clone())).unwrap(),
            token_accounts: self.token_accounts.iter().map(
                |t| t.format(&rpc_client).unwrap()).collect::<Vec<FormattedTokenAccount>>()
        });
    }

    //pub fn format_with_all_token_accounts(&self, rpc_client: &RpcClient) -> Result<FormattedUserConfig>{
    pub fn format(&self, rpc_client: &RpcClient) -> Result<FormattedUserConfig>{
        let signer_key = b58_to_pubkey(&self.signer_key);
        let keyed_token_accounts = rpc_client.get_token_accounts_by_owner(
            &signer_key, 
            TokenAccountsFilter::ProgramId(b58_to_pubkey(TOKEN_PROGRAM_ID))).unwrap();
        let token_accounts: Vec<FormattedTokenAccount> = keyed_token_accounts.iter().map(|t|{
            let key = b58_to_pubkey(&t.pubkey); // lazy implementation. 
            let mint = b58_to_pubkey(&rpc_client.get_token_account(&key).unwrap().unwrap().mint);
            FormattedTokenAccount{
                name: key.to_string(),
                key,
                mint,
            }
        }).collect();
        return Ok(FormattedUserConfig{
            signer_key,
            signer_keypair:  
                keypair::read_keypair_file(Path::new(&self.signer_keypair_path.clone())).unwrap(),
            token_accounts,
        });
    }
}

#[derive(Debug)]
pub struct FormattedUserConfig{
    pub signer_key: Pubkey,
    pub signer_keypair: Keypair,
    pub token_accounts: Vec<FormattedTokenAccount>,
}
impl FormattedUserConfig{
    pub fn gen_copy(&self) -> Self{
        let signer_keypair = Keypair::from_bytes(&self.signer_keypair.to_bytes()).unwrap();
        let token_accounts = self.token_accounts.iter()
            .map(|t| t.clone()).collect::<Vec<FormattedTokenAccount>>();
        Self{
            signer_key: self.signer_key.clone(),
            signer_keypair,
            token_accounts,

        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct TokenAccount{
    pub name: String,
    pub key: String,
}
impl TokenAccount{
    pub fn format(&self, rpc_client: &RpcClient) -> Result<FormattedTokenAccount>{
        return Ok(FormattedTokenAccount{
            name: self.name.clone(),
            key: crate::b58_to_pubkey(&self.key),
            mint: crate::spl_mint_key(&rpc_client, &self.key)?,
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct FormattedTokenAccount{
    pub name: String,
    pub key: Pubkey,
    pub mint: Pubkey,
}
//Write derive macros for these.
#[derive(Deserialize, PartialEq, Clone, Debug)]
pub struct TokenSwap{
    pub swap_program_name: String,
    pub swap_program: String,
    pub pools: Vec<Pool>
}
impl TokenSwap{
    pub fn format(&self, rpc_client: &RpcClient, user_config: &FormattedUserConfig) -> Result<FormattedTokenSwap>{
        return Ok(FormattedTokenSwap{
            swap_program_name: self.swap_program_name.clone(),
            swap_program: crate::b58_to_pubkey(&self.swap_program),
            pools: self.pools.iter().map(|p| p.format(rpc_client, user_config).unwrap()).collect::<Vec<FormattedPool>>(),
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct FormattedTokenSwap{
    pub swap_program_name: String,
    pub swap_program: Pubkey,
    pub pools: Vec<FormattedPool>
}

#[derive(Deserialize, PartialEq, Clone, Debug)]
pub struct OrcaTokenSwap{
    pub swap_program_name: String,
    pub swap_program: String,
    pub pools: Vec<Pool>
}
impl OrcaTokenSwap{
    pub fn format(&self, rpc_client: &RpcClient, user_config: &FormattedUserConfig) -> Result<FormattedOrcaTokenSwap>{
        return Ok(FormattedOrcaTokenSwap{
            swap_program_name: self.swap_program_name.clone(),
            swap_program: crate::b58_to_pubkey(&self.swap_program),
            pools: self.pools.iter().map(|p| p.format(rpc_client, user_config).unwrap()).collect::<Vec<FormattedPool>>(),
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct FormattedOrcaTokenSwap{
    pub swap_program_name: String,
    pub swap_program: Pubkey,
    pub pools: Vec<FormattedPool>,
}

#[derive(Deserialize, PartialEq, Clone, Debug)]
pub struct OrcaTokenSwapV2{
    pub swap_program_name: String,
    pub swap_program: String,
    pub pools: Vec<Pool>
}
impl OrcaTokenSwapV2{
    pub fn format(&self, rpc_client: &RpcClient, user_config: &FormattedUserConfig) -> Result<FormattedOrcaTokenSwapV2>{
        return Ok(FormattedOrcaTokenSwapV2{
            swap_program_name: self.swap_program_name.clone(),
            swap_program: crate::b58_to_pubkey(&self.swap_program),
            pools: self.pools.iter().map(|p| p.format(rpc_client, user_config).unwrap()).collect::<Vec<FormattedPool>>(),
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct FormattedOrcaTokenSwapV2{
    pub swap_program_name: String,
    pub swap_program: Pubkey,
    pub pools: Vec<FormattedPool>
}

#[derive(Deserialize, PartialEq, Clone, Debug)]
pub struct Pool{ // `Pool` == token_swap variant
    pub in_symbol: String,
    pub out_symbol: String,
    pub swap: String,
    pub swap_authority: String,
    pub signer: Option<String>,
    pub source_account_user: Option<String>,
    pub source_account_swap: String,
    pub destination_account_swap: String,
    pub destination_account_user: Option<String>,
    pub pool_mint: String,
    pub fee_account: String,
    pub token_program_id: Option<String>,
}

impl Pool{ 
    pub fn format(&self, rpc_client: &RpcClient, user_config: &FormattedUserConfig) -> Result<FormattedPool>{
        let in_symbol = self.in_symbol.clone();
        let out_symbol = self.out_symbol.clone();
        println!("TRACE --- loading {}/{}.", in_symbol, out_symbol);
        // todo: check that each spl_account_key is owned by user pubkey
        let source_account_user = crate::corresponding_user_wallet_mint(&rpc_client, 
            &user_config.token_accounts, b58_to_pubkey(&self.source_account_swap))?;
        let destination_account_user = crate::corresponding_user_wallet_mint(&rpc_client, 
            &user_config.token_accounts, b58_to_pubkey(&self.destination_account_swap))?;
        //println!("TRACE --- user_source_account: {}", bs58::decode(format!("{}", source_account_user)).into_vec().unwrap().len());
        //println!("TRACE --- user_source_account: {}", bs58::decode(format!("{}", destination_account_user)).into_vec().unwrap().len());
       /* TRACE.  
        println!("TRACE --- entering pool accounts");
        println!("{}", self.swap.chars().count());
        println!("{}", self.swap_authority.len());
        println!("{}", self.source_account_swap.len());
        println!("{}", self.destination_account_swap.len());
        println!("{}", self.pool_mint.len());
        println!("{}", self.fee_account.len());
        println!("{}", bs58::decode(TOKEN_PROGRAM_ID).into_vec().unwrap().len());
        */
        let account_keys = vec![
            b58_to_pubkey(&self.swap),
            b58_to_pubkey(&self.swap_authority),
            user_config.signer_key,
            source_account_user,
            b58_to_pubkey(&self.source_account_swap), //called twice
            b58_to_pubkey(&self.destination_account_swap), //called twice
            destination_account_user,
            b58_to_pubkey(&self.pool_mint),
            b58_to_pubkey(&self.fee_account),
            b58_to_pubkey(TOKEN_PROGRAM_ID),//&self.token_program_id.clone().unwrap_or(String::from(TOKEN_PROGRAM_ID))),
        ];
        
        let swap_program = rpc_client.get_account(&account_keys[0]).unwrap().owner; // -------------------------------- Wed do NOT want this to panic. Revise. Add is_acc_init.

        let account_metas = account_keys.iter().enumerate().map(|(i, k)|{
            let mut is_signer = false;
            let mut is_writable = false;
            match i{
                0 | 1 | 9 => {
                    is_signer = false;
                    is_writable = false;
                },
                2 => {
                    is_signer = true;
                    is_writable = true;
                },
                3..=8 => {
                    is_signer = false;
                    is_writable = true;
                },
                _ => (),
            };
            AccountMeta{ pubkey: *k, is_signer, is_writable }
        }).collect::<Vec<AccountMeta>>();
        
        println!("TRACE --- {}/{} loaded.\n", in_symbol, out_symbol);
        return Ok(FormattedPool{
            in_symbol, out_symbol, swap_program, account_keys, account_metas
        });
    }
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct FormattedPool{ // `Pool` == token_swap variant
    pub in_symbol: String,
    pub out_symbol: String,
    pub swap_program: Pubkey,
    pub account_keys: Vec<Pubkey>,
    pub account_metas: Vec<AccountMeta>,
}
impl FormattedPool{
    pub fn user_in_token_account(&self) -> Pubkey{
        return self.account_keys[3].clone();
    }
    
    pub fn user_out_token_account(&self) -> Pubkey{
        return self.account_keys[6].clone();
    }
}
/*
pub fn format_swap<T: NativeSwap, S: NativeSwap>(swap_obj: &T) -> Result<S>{
    match swap_obj {
        TokenSwap => (),
        OrcaTokenSwap => (),
        OrcaTokenSwapV2 => (),
        _ => (),
    };
    Ok(return FormattedTokenSwap{
        swap_program_name: swap_obj.swap_program_name,
        swap_program: crate::b58_to_pubkey(&format!("{}", swap_obj.swap_program)),
        pools: swap_obj.iter().map(|p| p.format()?).collect::<Vec<FormattedPool>>(),
    }); 
}
*/
