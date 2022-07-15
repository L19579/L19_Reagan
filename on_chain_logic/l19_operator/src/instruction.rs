use crate::{
    ProgramError,
    msg,
};

// Required accounts sort of hidden here because a large
// part of l19_operator is still active on chain. Easy to
// 'decode' with sprinkle of will power.

pub enum Instruction{
    //0. -- REMOVED
    //1. -- REMOVED
    //2. -- REMOVED
    //3. -- REMOVED
    //4. -- REMOVED 
    //5. -- REMOVED
    //6. -- REMOVED
    //7. -- REMOVED 
    //8. -- REMOVED
    //9. -- REMOVED
    //10.-- REMOVED
    //11.-- REMOVED
    //12.-- REMOVED
    //13.-- REMOVED
    //14.-- REMOVED
    //15.-- REMOVED
    //16.-- REMOVED
    //17.-- REMOVED
    StandardTwo,

    //0. -- REMOVED
    //1. -- REMOVED
    //2. -- REMOVED
    //3. -- REMOVED
    //4. -- REMOVED 
    //5. -- REMOVED
    //6. -- REMOVED
    //7. -- REMOVED 
    //8. -- REMOVED
    //9. -- REMOVED
    //10.-- REMOVED
    //11.-- REMOVED
    //12.-- REMOVED
    //13.-- REMOVED
    //14.-- REMOVED
    //15.-- REMOVED
    //16.-- REMOVED
    //17.-- REMOVED
    //18.-- REMOVED 
    //19.-- REMOVED
    //20.-- REMOVED
    //21.-- REMOVED
    //22.-- REMOVED
    //23.-- REMOVED
    //24.-- REMOVED
    //25.-- REMOVED
    StandardThree,
}

impl Instruction{
    pub fn unpack(input: &[u8]) -> Result<(Self, &[u8]), ProgramError>{
        let (tag, data) = match input.split_last(){
            Some(t_d) => t_d,
            None => {
                msg!("Instruction missing data");
                return Err(ProgramError::InvalidInstructionData);
            }
        };
        return Ok(
            match tag {
                0 => (Self::StandardTwo, data),
                1 => (Self::StandardThree, data),
                _ => return Err(ProgramError::InvalidInstructionData),
            }
        );
    }
}
