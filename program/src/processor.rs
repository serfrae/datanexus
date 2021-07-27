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

        Ok(())
    }

    fn process_set_params(
        program_id: Pubkey,
        accounts: &[AccountInfo],
        hash: [u8; 32],
        params: Params,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let authority = next_account_info(accounts_iter)?;
        let dataset_account = next_account_info(accounts_iter)?;

        let dataset_account_data = dataset_account.data.borrow_mut();
        let unpacked_dataset_data = DatasetState::unpack_from_slice(dataset_account_data)?;

        if authority.key != unpacked_dataset_data.owner {
            msg!("Incorrect Dataset Owner");
            return Err(ProgramError::InvalidArgument);
        }

        match params {
            Params::Init => {
                let DatasetState {
                    mut key,
                    mut value,
                    mut share_limit,
                    mut ref_data,
                    ..
                } = unpacked_dataset_data;

                key = Some(params.0);
                value = Some(params.1);
                share_limit = Some(params.2);
                ref_data = match params.3 {
                    Some(n) => Some(n),
                    None => None,
                };
            }
            Params::Key => unpacked_dataset_data.key = Some(params.0),
            Params::Value => unpacked_dataset_data.value = Some(params.0),
            Params::ShareLimit => unpacked_dataset_data.share_limit = Some(params.0),
            Params::ReferenceData => unpacked_dataset_data.ref_data = Some(params.0),
            _ => return Err(ProgramError::InvalidArgument),
        }

        unpacked_dataset_data.pack_into_slice(dataset_account_data);

        Ok(())
    }

    fn process_purchase_access(
        program_id: Pubkey,
        accounts: &[AccountInfo],
        hash: [u8; 32],
        amount: u64,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let user_authority = next_account_info(accounts_iter)?;
        let user_access_account = next_account_info(accounts_iter)?;
        let user_token_account = next_account_info(accounts_iter)?;
        let owner_authority = next_account_info(accounts_iter)?;
        let owner_token_account = next_account_info(accounts_iter)?;
        let dataset_account = next_account_info(accounts_iter)?;
        let token_program = next_account_info(accounts_iter)?;

        if token_program.key != spl_token::ID {
            msg!("Incorrect token program ID");
            return Err(ProgramError::IncorrectProgramId);
        }

        let transfer_ix = spl_token::instruction::transfer(
            token_program.key,
            user_token_account.key,
            owner_token_account.key,
            user_authority.key,
            &[user_authority.key],
            amount,
        )?;

        invoke(
            &transfer_ix,
            &[
                user_token_account.clone(),
                owner_token_account.clone(),
                user_authority.clone(),
                &[user_authority.clone()],
            ],
        )?;

        let user_access_account_data = user_access_account.data.borrow_mut();
        let unpacked_user_access_data = AccessState::unpack_from_slice(user_access_account_data)?;
        let dataset_account_data = dataset_account.data.borrow();
        let unpacked_dataset_data = DatasetState::unpack_from_slice(dataset_account_data)?;
        let new_access = AccessInfo {
            hash,
            key: unpacked_dataset_data.key,
            shared_from: None,
            share_limit: unpacked_dataset_data.share_limit,
        }
        .pack();

        Ok(())
    }

    fn process_share_access(
        program_id: Pubkey,
        accounts: &[AccountInfo],
        hash: [u8; 32],
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let user_authority = next_account_info(accounts_iter)?;
        let user_access_account = next_account_info(accounts_iter)?;
        let recipient_authority = next_account_info(accounts_iter)?;
        let recipient_access_account = next_account_info(accounts_iter)?;
        let dataset_account = next_account_info(accounts_iter)?;

        let dataset_account_data = dataset_account.data.borrow();
        let unpacked_dataset_data = DatasetState::unpack_from_slice(dataset_account_data)?;
        let user_access_data = user_access_account.data.borrow_mut();
        let unpacked_user_access_data = AccessState::unpack_from_slice(user_access_account_data)?;
        let recipient_access_data = recipient_access_account.data.borrow_mut();
        let unpacked_recipient_access_data = AccessState::unpack_from_slice(recipient_access_data)?;

        let new_access = AccessInfo {
            hash,
            key: unpacked_user_access_data.key,
            shared_from: user_authority.key,
            share_limit: 0,
        };

        unpacked_recipient_access_data.datasets.push(new_access);
        unpacked_recipient_access_data.pack_into_slice(recipient_access_data);

        unpacked_user_access_data.share_limit -= 1;
        unpacked_user_access_data.pack_into_slice(user_access_data);

        Ok(())
    }
}
