pub mod instruction;
pub mod entrypoint;
pub mod processor;

pub use {
    solana_program::{
        account_info::AccountInfo,
        entrypoint::ProgramResult,
        program_error::ProgramError,
        pubkey::Pubkey,
        msg,
    },
};

//Program ID added to prevent accidental calls but stopped
//short of impl code to save on compute units. 
pub const PROGRAM_ID: &str = "ADD_PROGRAM_ID";
pub const ALLOWED_KEYS: [&str; 6] = [
    "ADD_ALLOWED_CALLER_PUBKEYS", 
];
