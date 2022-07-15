use std::string::ToString;
use colored::Colorize;
use crate::Pubkey;

pub enum Error<'berlin>{
    UnableToBuildConfig,
    WalletVaultMintKeyMismatch,
    VaultBalanceZero,
    UnableToFindMint,
    TooManyInstructions,
    LegCountDisallowed,
    PoolQuantitiesNotUpdated,
    PoolsNotLinked,
    UnableToFetchTokenAccountBalance(&'berlin Pubkey),
}

impl<'berlin> ToString for Error<'berlin>{
    fn to_string(&self) -> String{
        match self{
            Error::UnableToBuildConfig => String::from("unable to build config"),
            Error::WalletVaultMintKeyMismatch => String::from("wallet vault mint key mismatch"),
            Error::VaultBalanceZero => String::from("one or more of pool vault balance returned is zero"),
            Error::UnableToFindMint => String::from("could not find mint"),
            Error::TooManyInstructions => String::from("too many instructions"),
            Error::LegCountDisallowed => String::from("leg count outside of allowed range"),
            Error::PoolQuantitiesNotUpdated => String::from("pool quantities not up to date"),
            Error::PoolsNotLinked => String::from("pools in proposed ArbPath are not linked"),
            Error::UnableToFetchTokenAccountBalance(p) => format!("Unable to fetch token account balance for: {}", p),
        }
    }
}

pub enum StatementType{
    Test,
    General,
    Success,
    BoldSuccess,
    Error,
    BoldError,
    Warning,
    BoldWarning,
    OpportunityFound,
    TransactionSuccessful,
    TransactionFailed,
}

pub fn show_statement(statement_type: StatementType, statement: &str){
    let mut output = match statement_type {
        StatementType::Test => format!("{:<16} : ", "general".cyan()),
        StatementType::General => format!("{:<16} : ", "general".bright_blue()),
        StatementType::Success => format!("{:<16} : ", "success".bright_green()),
        StatementType::BoldSuccess => format!("{:<16} : ", "success".bright_green().bold()),
        StatementType::Error => format!("{:<16} : ", "error".bright_red()),
        StatementType::BoldError => format!("{:<16} : ", "error".bright_red().bold()),
        StatementType::Warning => format!("{:<16} : ", "warning".yellow()),
        StatementType::BoldWarning => format!("{:<16} : ", "warning".yellow().bold()),
        StatementType::OpportunityFound => format!("{:<16} : ", "opportunity found".bright_green()),
        StatementType::TransactionSuccessful => format!("{:<16} : ", "tx successful".bright_green()),
        StatementType::TransactionFailed => format!("{:<16} : ", "tx failed".bright_red()),
    };
    output.push_str(statement);
    println!("{}", output);
}
