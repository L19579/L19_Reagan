use solana_program::{
    account_info::{
        AccountInfo,
        next_account_info,
    },
    entrypoint::ProgramResult,
    program_error::ProgramError,
    msg,
    pubkey::Pubkey,
    program::invoke,
    instruction,
};

use crate::{
    PROGRAM_ID,
    instruction::Instruction
};

pub struct Processor;

impl Processor{
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], 
    instruction_data: &[u8]) -> ProgramResult{
        let (instruction, data) = Instruction::unpack(instruction_data).unwrap();
        match instruction{
            Instruction::StandardTwo => {
                Self::process_standard_two(accounts, data) 
            }
            Instruction::StandardThree => {
                Self::process_standard_three(accounts, data)
            }
        }
    }
    // Data management is a bit loose.
    fn process_standard_two(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult{
        let account_info_iter = &mut accounts.iter();
        let caller_info = next_account_info(account_info_iter).unwrap();
        if !caller_info.is_signer{
            return Err(ProgramError::MissingRequiredSignature);
        }       
        /* No longer in use.
        let (settlement_ui_decimal, data) = data.split_last().unwrap();
        let (first_transfer_ui_decimal, data) = data.split_last().unwrap();
        */
        //check that program's being called from valid keys here.
        let token_program_id_info = next_account_info(account_info_iter).unwrap();
        /*
        let amm_program_one_info = next_account_info(account_info_iter).unwrap();
        let amm_program_two_info = next_account_info(account_info_iter).unwrap();
        */
        _ = next_account_info(account_info_iter).unwrap();
        _ = next_account_info(account_info_iter).unwrap();
        
        let settlement_token_account_info = next_account_info(account_info_iter).unwrap();
        let first_transfer_token_account_info = next_account_info(account_info_iter).unwrap();

        let account_metas: Vec<instruction::AccountMeta> = accounts.iter().enumerate().map(|(i, a)|{
            let mut is_signer = false;
            let mut is_writable = false;

            match i {
                0 => {
                    is_signer = true;
                    is_writable = true;
                },
                4 | 5 | 8..=11 | 14..=17 => {
                    is_writable = true;
                }
                _ => (),
            };
            instruction::AccountMeta{
                pubkey: *a.key,
                is_signer,
                is_writable
            }
        }).collect();
        
        let caller = &account_metas[0];
        let token_program_id = &account_metas[1];
        let amm_program_one = &account_metas[2];
        let amm_program_two = &account_metas[3];
        
        let settlement_token_account = &account_metas[4];
        let first_transfer_token_account = &account_metas[5];

        let req_accs_and_data = [
            (
                amm_program_one,
                vec![
                    next_account_info(account_info_iter).unwrap().clone(),  
                    next_account_info(account_info_iter).unwrap().clone(),  
                    caller_info.clone(),                                    
                    settlement_token_account_info.clone(),                  
                    next_account_info(account_info_iter).unwrap().clone(),  
                    next_account_info(account_info_iter).unwrap().clone(),   
                    first_transfer_token_account_info.clone(),              
                    next_account_info(account_info_iter).unwrap().clone(),   
                    next_account_info(account_info_iter).unwrap().clone(),   
                    token_program_id_info.clone(),                          
                ],
                vec![
                    account_metas[6].clone(),
                    account_metas[7].clone(),
                    caller.clone(),                                         
                    settlement_token_account.clone(),                       
                    account_metas[8].clone(),
                    account_metas[9].clone(),
                    first_transfer_token_account.clone(),                   
                    account_metas[10].clone(),
                    account_metas[11].clone(),
                    token_program_id.clone(),                               
                ],
                &data[..17]
            ),
            (
                amm_program_two,
                vec![
                    next_account_info(account_info_iter).unwrap().clone(),  
                    next_account_info(account_info_iter).unwrap().clone(),  
                    caller_info.clone(),                                    
                    first_transfer_token_account_info.clone(),              
                    next_account_info(account_info_iter).unwrap().clone(),  
                    next_account_info(account_info_iter).unwrap().clone(),   
                    settlement_token_account_info.clone(),                  
                    next_account_info(account_info_iter).unwrap().clone(),   
                    next_account_info(account_info_iter).unwrap().clone(),   
                    token_program_id_info.clone(),                          
                ],
                vec![
                    account_metas[12].clone(),
                    account_metas[13].clone(),
                    caller.clone(),                                         // transfer authority
                    first_transfer_token_account.clone(),                   // user destination
                    account_metas[14].clone(),
                    account_metas[15].clone(),
                    settlement_token_account.clone(),                       // user source
                    account_metas[16].clone(),
                    account_metas[17].clone(),
                    token_program_id.clone(),                               // token program
                ],
                &data[17..34]
            )
        ];
        let initial_settlement_account_balance = token_account_balance(&settlement_token_account_info);
        let initial_first_transfer_account_balance = token_account_balance(&first_transfer_token_account_info);
        
        //instructions grouped, in_amounts modified after zero index.
        let mut swap_instructions: Vec<instruction::Instruction> = req_accs_and_data.iter().map(|a_d|{
            instruction::Instruction::new_with_bytes(a_d.0.pubkey, &a_d.3, a_d.2.clone())
        }).collect();

        //first swap
        let amount_in: u64 = data.get(1..9) 
            .and_then(|i_a| i_a.try_into().ok())
            .map(|a_i| u64::from_le_bytes(a_i)).unwrap();
        swap_instructions[0].data = generate_next_data(amount_in);
        invoke(
            &swap_instructions[0], 
            &req_accs_and_data[0].1.clone()[..],
        ).unwrap();
        
        // second swap
        let current_first_transfer_account_balance = token_account_balance(&first_transfer_token_account_info);
        let first_transfer_balance_difference = current_first_transfer_account_balance as i64 - initial_first_transfer_account_balance as i64;
        /*
        if first_transfer_balance_difference < 0  { // <-- this is pointless
            msg!("Negative pnl");
            return Err(ProgramError::Custom(0));
        }
        */
        //let amount_in = token_account_balance(&first_transfer_token_account_info);
        let amount_in = first_transfer_balance_difference as u64;
        swap_instructions[1].data = generate_next_data(amount_in);
        
        invoke(
            &swap_instructions[1],
            &req_accs_and_data[1].1.clone()[..],
        ).unwrap();

        // Checking for positive pnl
        let final_settlement_account_balance = token_account_balance(&settlement_token_account_info);
        if (final_settlement_account_balance as i64 - initial_settlement_account_balance as i64) < 0  {
            msg!("bohoo");
            return Err(ProgramError::Custom(0));
        }
        
        msg!("got em");
        return Ok(());
    }
    
    fn process_standard_three(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult{
        let account_info_iter = &mut accounts.iter();
        let caller_info = next_account_info(account_info_iter).unwrap();
        if !caller_info.is_signer{
            return Err(ProgramError::MissingRequiredSignature);
        }       
        
        /* No long in use.
        let (settlement_ui_decimal, data) = data.split_last().unwrap();
        let (first_transfer_ui_decimal, data) = data.split_last().unwrap();
        let (second_transfer_ui_decimal, data) = data.split_last().unwrap();
        */

        //check that program's being called from valid keys here.
        let token_program_id_info = next_account_info(account_info_iter).unwrap();
        let amm_program_one_info = next_account_info(account_info_iter).unwrap();
        let amm_program_two_info = next_account_info(account_info_iter).unwrap();
        let amm_program_three_info = next_account_info(account_info_iter).unwrap();
        
        let settlement_token_account_info = next_account_info(account_info_iter).unwrap();
        let first_transfer_token_account_info = next_account_info(account_info_iter).unwrap();
        let second_transfer_token_account_info = next_account_info(account_info_iter).unwrap();

        let account_metas: Vec<instruction::AccountMeta> = accounts.iter().enumerate().map(|(i, a)|{
            let mut is_signer = false;
            let mut is_writable = false;

            match i {
                0 => {
                    is_signer = true;
                    is_writable = true;
                },
                5 | 6 | 7 | 10..=13 | 16..=19 | 22..=25 => {
                    is_writable = true;
                }
                _ => {()},
            };
            instruction::AccountMeta{
                pubkey: *a.key,
                is_signer,
                is_writable
            }
        }).collect();
        
        let caller = &account_metas[0];
        let token_program_id = &account_metas[1];
        let amm_program_one = &account_metas[2];
        let amm_program_two = &account_metas[3];
        let amm_program_three = &account_metas[4];
        
        let settlement_token_account = &account_metas[5];
        let first_transfer_token_account = &account_metas[6];
        let second_transfer_token_account = &account_metas[7];

        let req_accs_and_data = [
            (
                amm_program_one,
                vec![
                    next_account_info(account_info_iter).unwrap().clone(),  
                    next_account_info(account_info_iter).unwrap().clone(),  
                    caller_info.clone(),                                    
                    settlement_token_account_info.clone(),                  
                    next_account_info(account_info_iter).unwrap().clone(),  
                    next_account_info(account_info_iter).unwrap().clone(),   
                    first_transfer_token_account_info.clone(),              
                    next_account_info(account_info_iter).unwrap().clone(),   
                    next_account_info(account_info_iter).unwrap().clone(),   
                    token_program_id_info.clone(),                          
                ],
                vec![
                    account_metas[8].clone(),
                    account_metas[9].clone(),
                    caller.clone(),                                         
                    settlement_token_account.clone(),                       
                    account_metas[10].clone(),
                    account_metas[11].clone(),
                    first_transfer_token_account.clone(),                   
                    account_metas[12].clone(),
                    account_metas[13].clone(),
                    token_program_id.clone(),                               
                ],
                &data[..17]
            ),
            (
                amm_program_two,
                vec![
                    next_account_info(account_info_iter).unwrap().clone(),  
                    next_account_info(account_info_iter).unwrap().clone(),  
                    caller_info.clone(),                                    
                    first_transfer_token_account_info.clone(),              
                    next_account_info(account_info_iter).unwrap().clone(),  
                    next_account_info(account_info_iter).unwrap().clone(),   
                    second_transfer_token_account_info.clone(),             
                    next_account_info(account_info_iter).unwrap().clone(),   
                    next_account_info(account_info_iter).unwrap().clone(),   
                    token_program_id_info.clone(),                          
                ],
                vec![
                    account_metas[14].clone(),
                    account_metas[15].clone(),
                    caller.clone(),                                         
                    first_transfer_token_account.clone(),                   
                    account_metas[16].clone(),
                    account_metas[17].clone(),
                    second_transfer_token_account.clone(),                  
                    account_metas[18].clone(),
                    account_metas[19].clone(),
                    token_program_id.clone(),                               
                ],
                &data[17..34]
            ),
            (
                amm_program_three,
                vec![
                    next_account_info(account_info_iter).unwrap().clone(),
                    next_account_info(account_info_iter).unwrap().clone(),
                    caller_info.clone(),                                  
                    second_transfer_token_account_info.clone(),           
                    next_account_info(account_info_iter).unwrap().clone(),
                    next_account_info(account_info_iter).unwrap().clone(), 
                    settlement_token_account_info.clone(),                
                    next_account_info(account_info_iter).unwrap().clone(), 
                    next_account_info(account_info_iter).unwrap().clone(), 
                    token_program_id_info.clone(),                      
                ],
                vec![
                    account_metas[20].clone(),
                    account_metas[21].clone(),
                    caller.clone(),                                     
                    second_transfer_token_account.clone(),              
                    account_metas[22].clone(),
                    account_metas[23].clone(),
                    settlement_token_account.clone(),                   
                    account_metas[24].clone(),
                    account_metas[25].clone(),
                    token_program_id.clone(),                               
                ],
                &data[34..51]
            )
        ];
        let initial_settlement_account_balance = token_account_balance(&settlement_token_account_info);
        let initial_first_transfer_account_balance = token_account_balance(&first_transfer_token_account_info);
        let initial_second_transfer_account_balance = token_account_balance(&second_transfer_token_account_info);

        // grouped instructions, in_amounts modified after zero index 
        let mut swap_instructions: Vec<instruction::Instruction> = req_accs_and_data.iter().map(|a_d|{
            instruction::Instruction::new_with_bytes(a_d.0.pubkey, &a_d.3, a_d.2.clone())
        }).collect();

        //first swap
        let amount_in: u64 = data.get(1..9)
            .and_then(|i_a| i_a.try_into().ok())
            .map(|a_i| u64::from_le_bytes(a_i)).unwrap();
        swap_instructions[0].data = generate_next_data(amount_in);
        invoke(
            &swap_instructions[0].clone(),
            &req_accs_and_data[0].1.clone()[..],
        ).unwrap();
        
        // second swap
        let current_first_transfer_account_balance = token_account_balance(&first_transfer_token_account_info);
        let first_transfer_balance_difference = current_first_transfer_account_balance as i64 - initial_first_transfer_account_balance as i64;
        //let amount_in = token_account_balance(&first_transfer_token_account_info);
        let amount_in = first_transfer_balance_difference as u64;
        swap_instructions[1].data = generate_next_data(amount_in);
        invoke(
            &swap_instructions[1].clone(),
            &req_accs_and_data[1].1.clone()[..],
        ).unwrap();
       
        // third swap
        let current_second_transfer_account_balance = token_account_balance(&second_transfer_token_account_info);
        let second_transfer_balance_difference = current_second_transfer_account_balance as i64 - initial_second_transfer_account_balance as i64;
        //let amount_in = token_account_balance(&second_transfer_token_account_info);
        let amount_in = second_transfer_balance_difference as u64;
        swap_instructions[2].data = generate_next_data(amount_in);
        invoke(
            &swap_instructions[2],
            &req_accs_and_data[2].1.clone()[..],
        ).unwrap();
        
        //checking for positive pnl
        let final_settlement_account_balance = token_account_balance(&settlement_token_account_info);
        if (final_settlement_account_balance as i64 - initial_settlement_account_balance as i64) < 0  {
            msg!("bohoo");
            return Err(ProgramError::Custom(0));
        }
        
        msg!("got em");
        return Ok(());
    }
}

fn token_account_balance(token_account_info: &AccountInfo) -> u64{
    let mut token_account_balance_bytes = [0u8; 8];
    token_account_balance_bytes.copy_from_slice(&token_account_info.clone()
        .data.borrow()[64..72]);
    return u64::from_le_bytes(token_account_balance_bytes);
}

///amount_out defaults to 0u64
fn generate_next_data(amount_in: u64) -> Vec<u8>{
    let mut data = Vec::<u8>::new();
    data.push(1);
    data.extend_from_slice(&amount_in.to_le_bytes());
    data.extend_from_slice(&0u64.to_le_bytes());
    return data;
}
