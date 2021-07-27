use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
};

use spl_token::state::Account;

use crate::{error::DataNexusError, instruction::DataNexusInstruction, state::*};

pub struct Processor {}

impl Processor {
    pub fn process_instruction(
        program_id: Pubkey,
        accounts: &[AccountInfo],
        data: &[u8],
    ) -> ProgramResult {
        let instruction = DataNexusInstruction::unpack(data)?;

        match instruction {
            DataNexusInstruction::InitUserAccount(AccountType::Owner) => {
                Self::process_init_owner_account(program_id, accounts)
            }
            DataNexusInstruction::InitUserAccount(AccountType::Access) => {
                Self::process_init_access_account(program_id, accounts)
            }
            DataNexusInstruction::InitDataAccount { hash } => {
                Self::process_init_data_account(program_id, accounts, hash)
            }
            DataNexusInstruction::SetDataParams { hash, params } => {
                Self::process_set_params(program_id, accounts, hash, params)
            }
            DataNexusInstruction::PurchaseAccess { hash, amount } => {
                Self::process_purchase_access(program_id, accounts, hash, amount)
            }
            DataNexusInstruction::ShareAccess { hash } => {
                Self::process_share_access(program_id, accounts, hash)
            }
            _ => return Err(DataNexusError::InvalidInstruction.into()),
        }

        Ok(())
    }

    fn process_init_owner_account(program_id: Pubkey, accounts: &[AccountInfo]) -> ProgramResult {}

    fn process_init_access_account(program_id: Pubkey, accounts: &[AccountInfo]) -> ProgramResult {}

    fn process_init_dataset_account(
        program_id: Pubkey,
        accounts: &[AccountInfo],
        hash: [u8; 32],
    ) -> ProgramResult {
    }

    fn process_set_params(
        program_id: Pubkey,
        accounts: &[AccountInfo],
        hash: [u8; 32],
        params: Params,
    ) -> ProgramResult {
    }

    fn process_purchase_access(
        program_id: Pubkey,
        accounts: &[AccountInfo],
        hash: [u8; 32],
        amount: u64,
    ) -> ProgramResult {
    }

    fn process_share_access(
        program_id: Pubkey,
        accounts: &[AccountInfo],
        hash: [u8; 32],
    ) -> ProgramResult {
    }
}
