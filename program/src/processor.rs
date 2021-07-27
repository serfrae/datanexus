use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction::create_account,
    sysvar::rent::Rent,
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
            DataNexusInstruction::InitUserAccount(account_type) => {
                Self::process_init_user_account(program_id, accounts, account_type)
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

    fn process_init_user_account(
        program_id: Pubkey,
        accounts: &[AccountInfo],
        account_type: AccountType,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let payer = next_account_info(accounts_iter)?;
        let user_account = next_account_info(accounts_iter)?;
        let system_program = next_account_info(accounts_iter)?;
        let rent = Rent::get();

        let state_size = if account_type == AccountType::Owner {
            OwnerState::LEN
        } else {
            AccessState::LEN
        };

        let create_account_ix = create_account(
            payer.key,
            user_account.key,
            rent.mininum_balance(state_size),
            state_size,
            program_id,
        );

        invoke(
            &create_account_ix,
            &[system_program.clone(), payer.clone(), owner_account.clone()],
        )?;

        let account_data = user_account.data.borrow_mut();
        let is_initialized = true;
        let pointer = None;
        let datasets: [Option<_>; 128] = [None; 128];

        if account_type == AccountType::Owner {
            OwnerState {
                is_initialized,
                pointer,
                datasets,
            }
            .pack_into_slice(account_data);
        } else {
            AccessState {
                is_initialized,
                pointer,
                datasets,
            }
            .pack_into_slice(account_data);
        }

        Ok(())
    }

    fn process_init_dataset_account(
        program_id: Pubkey,
        accounts: &[AccountInfo],
        hash: [u8; 32],
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let authority = next_account_info(accounts_iter)?;
        let owner_account = next_account_info(accounts_iter)?;
        let dataset_account = next_account_info(accounts_iter)?;
        let system_program = next_account_info(accounts_iter)?;
        let rent = Rent::get();

        let create_account_ix = create_account(
            authority.key,
            dataset_account.key,
            rent.minimum_balance(DatasetState::LEN),
            DatasetState::LEN,
            program_id,
        );

        invoke(
            &create_account,
            &[authority.clone(), dataset_account.clone()],
        )?;

        let owner_account_data = owner_account.data.borrow_mut();
        let unpacked_owner_account_data = OwnerState::unpack_from_slice(owner_account_data)?;

        unpacked_owner_account_data
            .datasets
            .push(dataset_account.key);

        unpacked_owner_account_data.pack_into_slice(owner_account_data);

        let dataset_account_data = dataset_account.data.borrow_mut();
        let is_initialized = true;
        let owner = authority.key;
        let key = None;
        let value = None;
        let share_limit = None;

        DatasetState {
            is_initialized,
            owner,
            hash,
            key,
            value,
            share_limit,
        }
        .pack_into_slice(dataset_account_data);
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
