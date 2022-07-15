use solana_program::entrypoint;
use crate::{
    processor::Processor,
    AccountInfo,
    ProgramResult, //entrypoint seperated because of namespace conflict.
    Pubkey, 
};

entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    return Processor::process(
            program_id,
            accounts,
            instruction_data,
    )
}
