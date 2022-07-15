pub use crate::swap_repo::{
    curve::{
        fees::Fees,
        base::*,
        calculator::*, //{CurveCalculator, SwapWithoutFeeResult, TradeDirection}
        constant_product::ConstantProductCurve,
        offset::OffsetCurve,
        stable::StableCurve,
    },
};
pub use {
    bs58,
    toml,
    serde,
    serde_derive,
    rand::Rng,
    bincode,
    anyhow::{anyhow, Result},
    rand::{self, rngs::OsRng},
    chrono::{self, prelude::*, DateTime}, 
    std::{
        mem,
        thread,
        ops::Deref,
        boxed::Box,
        fmt::Display,
        time::Duration,
        iter::zip,
        path::Path,
        panic::catch_unwind,
        io::{ prelude::*, BufReader},
        fs::{File, OpenOptions},
        sync::{
            Mutex, Arc, RwLock, mpsc, 
            RwLockReadGuard, RwLockWriteGuard
        }, //drop mutex
    },
    crossbeam_channel::{
        bounded,
        unbounded,
        Sender,
        Receiver,
    },
    solana_sdk::{
        self,
        instruction,
        program_pack::{Pack, Sealed},
        pubkey::Pubkey,
        signer::keypair::Keypair,
        system_instruction::*, // create_account
        instruction::{Instruction, AccountMeta},
        transaction::Transaction,
        message::Message,
        signature::*,
        commitment_config::CommitmentConfig,
    },
    solana_client::{
        rpc_client::RpcClient, 
        rpc_request::TokenAccountsFilter,
        rpc_config::*, //RpcSendTransactionConfig
    },
    spl_token::instruction as spl_instruction,
    solana_account_decoder::parse_token::{
        UiTokenAmount,
        get_token_account_mint,
    },
    spl_associated_token_account::instruction as spl_associated_account_instruction,
};

pub const RPC_SSO_LINK: &str = "https://ssc-dao.genesysgo.net";
pub const RPC_SOLANA_LINK: &str = "https://api.mainnet-beta.solana.com";
pub const RPC_SERUM_LINK: &str = "https://solana-api.projectserum.com";

pub const CONFIG_PATH: &str = "**INSERT_DIRECT_PATH/L19_Reagan/data/config_paths.toml";

pub const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

pub const L19_PROGRAM_ID: &str = "YOUR_ON_CHAIN_PROGRAM_ID_HERE"; //required for capture_local_local_program()

pub const MAX_FEE: f64 = 0.0034;
pub const MAX_SLIPPAGE: f64 = 0.0007;
pub const MAX_CHANNEL_MESSAGES: u8 = 100;
