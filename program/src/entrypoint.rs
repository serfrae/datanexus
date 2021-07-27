use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};

use crate::{error::DataNexusError, processor::Processor};

entrypoint!(datanexus_entrypoint);

pub fn datanexus_entrypoint(
    program_id: Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    msg!("DataNexus Entrypoint");

    if let Err(error) = Processor::process_instruction(program_id, accounts, data) {
        print.error::<DataNexusError>();
        return Err(error);
    }

    Ok(())
}
