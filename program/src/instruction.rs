use solana_program::{
    instruction::{AccountMeta, Instruction},
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::error::DataNexusError;

enum AccountType {
    DatasetIndex,
    AccessIndex,
    Dataset([u8; 32]),
    Access([u8; 32]),
}

enum Params {
    Init(
        Self::Key,
        Self::Value,
        Self::ShareLimit,
        Option<Self::ReferenceData>,
    ),
    Key([u8; 32]),
    Value(u64),
    ShareLimit(u16),
    ReferenceData([u8; 32]),
}

pub enum DataNexusInstruction {
    /// Initialize Dataset Account
    ///
    /// Accounts expected:
    ///
    /// Distinct Payer account required when sharing access
    /// As the sharer will pay for initializing the recipient's
    /// AccessIndex and Access accounts
    ///
    /// Index Accounts:
    /// `[w,s]` Payer
    /// `[w]` Authority
    /// `[]` System Program
    ///
    /// Dataset/Access:
    /// `[w,s]` Payer
    /// `[w]` Authority
    /// `[w]` Dataset Index Account
    /// `[w]` Dataset Account
    /// `[]` System Program
    InitAccount(AccountType),

    /// Write Dataset Parameters to Buffer
    ///
    /// Accounts expected:
    ///
    /// `[w,s]` Authority
    /// `[w]` Dataset Account
    SetDataParams { hash: [u8; 32], params: Params },

    /// Purchase Dataset Access
    ///
    /// Accounts expected:
    ///
    /// `[w,s]` User Authority
    /// `[w]` User Access Index
    /// `[w]` User Access Account
    /// `[w]` User Token Account
    /// `[w]` Owner Account
    /// `[w]` Owner Token Account
    /// `[]` Dataset Account
    /// `[]` Token Program
    PurchaseAccess { hash: [u8; 32], amount: u64 },

    /// Share Dataset Access
    ///
    /// Accounts expected:
    ///
    /// `[w,s]` User Authority
    /// `[w]` User Access Index
    /// `[w]` User Access Account
    /// `[w]` Recipient Authority
    /// `[w]` Recipient Access Index
    /// `[w]` Recipient Access Account
    /// `[]` Dataset Account
    ShareAccess { hash: [u8; 32] },
}

impl DataNexusInstruction {
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());

        match self {
            Self::InitAccount(_) => {
                buf.push(0);
                match self {
                    Self::InitAccount(AccountType::DatasetIndex()) => buf.push(0),
                    Self::InitAccount(AccountType::AccessIndex()) => buf.push(1),
                    Self::InitAccount(AccountType::Dataset(hash)) => {
                        buf.push(2);
                        buf.extend_from_slice(hash);
                    }
                    Self::InitAccount(AccountType::Access(hash)) => {
                        buf.push(3);
                        buf.extend_from_slice(hash);
                    }
                    _ => return Err(InvalidInstruction.into()),
                }
            }
            Self::SetDataParams { hash, params } => {
                buf.push(2);
                buf.extend_from_slice(hash);
                match params {
                    Params::Init(key, value, share_limit, ref_data) => {
                        buf.push(0);
                        buf.extend_from_slice(key);
                        buf.extend_from_slice(value.to_le_bytes());
                        buf.extend_from_slice(share_limit.to_le_bytes());
                        match ref_data {
                            Some(d) => buf.extend_from_slice(d.to_bytes()),
                            None => buf.extend_from_slice([0u8; 32]),
                        }
                    }
                    Params::Key(k) => {
                        buf.push(1);
                        buf.extend_from_slice(k);
                    }
                    Params::Value(v) => {
                        buf.push(2);
                        buf.extend_from_slice(v.to_le_bytes());
                    }
                    Params::ShareLimit(n) => {
                        buf.push(3);
                        buf.extend_from_slice(n.to_le_bytes());
                    }
                    Params::ReferenceData(pk) => {
                        buf.push(4);
                        buf.extend_from_slice(pk.to_bytes());
                    }
                    _ => return Err(InvalidInstruction.into()),
                }
            }
            Self::PurchaseAccess { hash, amount } => {
                buf.push(3);
                buf.extend_from_slice(hash);
                buf.extend_from_slice(amount.to_le_bytes());
            }
            Self::ShareAccess { hash } => {
                buf.push(4);
                buf.extend_from_slice(hash);
            }
            _ => return Err(InvalidInstruction.into()),
        }
        buf
    }

    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = data.split_first().ok_or(InvalidInstruction)?;

        match tag {
            0 => Ok(match rest {
                0 => Self::InitAccount(AccountType::DatasetIndex),
                1 => Self::InitAccount(AccountType::AccessIndex),
                2 => Self::InitAccount(AccountType::DatasetAccount(
                    rest.get(1..33)
                        .and_then(|slice| slice.try_into().ok())
                        .unwrap(),
                )),
                3 => Self::InitAccount(AccountType::AccessAccount(
                    rest.get(1..33)
                        .and_then(|slice| slice.try_into().ok())
                        .unwrap(),
                )),
                _ => return Err(InvalidInstruction.into()),
            }),
            1 => {
                let (tag, rest) = rest.split_first().ok_or(InvalidInstruction)?;
                let params = match tag {
                    0 => {
                        let key = Params::Key(rest.get(..32).unwrap());
                        let value = Params::Value(
                            rest.get(32..40)
                                .and_then(|slice| slice.try_into().ok())
                                .map(u64::from_le_bytes)
                                .ok_or(InvalidInstruction)?,
                        );
                        let share_limit = Params::ShareLimit(
                            rest.get(40..42)
                                .and_then(|slice| slice.try_into().ok())
                                .map(u16::from_le_bytes)
                                .ok_or(InvalidInstruction)?,
                        );
                        let ref_data = Params::ReferenceData(
                            rest.get(42..50)
                                .and_then(|slice| slice.try_into().ok())
                                .map(Pubkey::new)
                                .ok_or(InvalidInstruction)?,
                        );
                        Params::Init(key, value, share_limit, ref_data)
                    }
                    1 => Params::Key(rest.get(..).unwrap()),
                    2 => Params::Value(
                        rest.get(..)
                            .and_then(|slice| slice.try_into().ok())
                            .map(u64::from_le_bytes)
                            .ok_or(InvalidInstruction)?,
                    ),
                    3 => Params::ShareLimit(
                        rest.get(..)
                            .and_then(|slice| slice.try_into().ok())
                            .map(u16::from_le_bytes)
                            .ok_or(InvalidInstruction)?,
                    ),
                    4 => Params::ReferenceData(
                        rest.get(..)
                            .and_then(|slice| slice.try_into().ok())
                            .unwrap(),
                    ),
                    _ => return Err(InvalidInstruction.into()),
                };
                Ok(Self::SetDataParams { hash, params })
            }
            2 => {
                let hash = rest
                    .get(..32)
                    .and_then(|slice| slice.try_into().ok())
                    .unwrap();
                let amount = rest
                    .get(32..40)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                Ok(Self::PurchaseAccess { hash, amount })
            }
            3 => Ok(Self::ShareAccess {
                hash: rest
                    .get(..)
                    .and_then(|slice| slice.try_into().ok())
                    .unwrap(),
            }),
            _ => return Err(InvalidInstruction.into()),
        }
    }
}

/// Creates an `InitAccount` instruction
pub fn init_account(
    program_id: Pubkey,
    payer: Option<Pubkey>,
    authority: Pubkey,
    new_account: Pubkey,
    index_account: Option<Pubkey>,
    system_program: Pubkey,
    account_type: AccountType,
    hash: Option<[u8; 32]>,
) -> Result<Instruction, ProgramError> {
    let (accounts, data) = match account_type {
        AccountType::DatasetIndex => init_dataset_index(authority, new_account, system_program),
        AccountType::AccessIndex => init_access_index(
            if let Some(n) = payer { n } else { authority },
            authority,
            new_account,
            system_program,
        ),
        AccountType::Dataset(_) => init_dataset_account(
            authority,
            index_account.unwrap(),
            new_account,
            system_program,
            hash.unwrap(),
        ),
        AccountType::Access(_) => init_access_account(
            if let Some(n) = payer { n } else { authority },
            authority,
            index_account.unwrap(),
            new_account,
            system_program,
            hash.unwrap(),
        ),
        _ => unreachable!(),
    };

    Ok(Instruction {
        program_id,
        accounts,
        data,
    })
}

fn init_dataset_index(
    authority: Pubkey,
    dataset_index: Pubkey,
    system_program: Pubkey,
) -> (
    Vec<AccountMeta>,
    DataNexusInstruction::InitAccount(AccountType::DatasetIndex),
) {
    let accounts = vec![
        AccountMeta::new(authority, true),
        AccountMeta::new(dataset_index, true),
        AccountMeta::new_readonly(system_program, false),
    ];

    let data = DataNexusInstruction::InitAccount(AccountType::DatasetIndex).pack();

    (accounts, data)
}

fn init_access_index(
    payer: Pubkey,
    authority: Pubkey,
    access_index: Pubkey,
    system_program: Pubkey,
) -> (
    Vec<AccountMeta>,
    DataNexusInstruction::InitAccount(AccountType::AccessIndex),
) {
    let accounts = vec![
        AccountMeta::new(payer, true),
        AccountMeta::new(authority, false),
        AccountMeta::new(access_index, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let data = DataNexusInstruction::InitAccount(AccountType::AccessIndex).pack();

    (accounts, data)
}

fn init_dataset_account(
    authority: Pubkey,
    dataset_account: Pubkey,
    dataset_index: Pubkey,
    system_program: Pubkey,
    hash: [u8; 32],
) -> (
    Vec<AccountMeta>,
    DataNexusInstruction::InitAccount(AccountType::Dataset([u8; 32])),
) {
    let accounts = vec![
        AccountMeta::new(authority, true),
        AccountMeta::new(dataset_account, false),
        AccountMeta::new(dataset_index, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let data = DataNexusInstruction::InitAccount(AccountType::Dataset(hash)).pack();

    (accounts, data)
}

fn init_access_account(
    payer: Pubkey,
    authority: Pubkey,
    access_account: Pukey,
    access_index: Pubkey,
    system_program: Pubkey,
    hash: [u8; 32],
) -> (
    Vec<AccountMeta>,
    DataNexusInstruction::InitAccount(AccountType::Access([u8; 32])),
) {
    let accounts = vec![
        AccountMeta::new(payer, true),
        AccountMeta::new(authority, false),
        AccountMeta::new(access_account, false),
        AccountMeta::new(access_index, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let data = DataNexusInstruction::InitAccount(AccountType::Access(hash)).pack();

    (accounts, data)
}

/// Creates a `SetDataParams` instruction
pub fn set_data_params(
    program_id: Pubkey,
    authority: Pubkey,
    dataset_account: Pubkey,
    hash: [u8; 32],
    params: Params,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(authority, true),
        AccountMeta::new(owner_account, false),
        AccountMeta::new(dataset_account, false),
    ];

    let data = DataNexusInstruction::SetDataParams { hash, params }.pack();

    Ok(Instruction {
        program_id,
        accounts,
        data,
    })
}

/// Creates a `PurchaseAccess` instruction
pub fn purchase_access(
    program_id: Pubkey,
    user_authority: Pubkey,
    user_access_index: Pubkey,
    user_access_account: Pubkey,
    user_token_account: Pubkey,
    owner_authority: Pubkey,
    owner_token_account: Pubkey,
    dataset_account: Pubkey,
    token_program: Pubkey,
    hash: [u8; 32],
    amount: u64,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(user_authority, true),
        AccountMeta::new(user_access_index, false),
        AccountMeta::new(user_access_account, false),
        AccountMeta::new(user_token_account, false),
        AccountMeta::new(owner_authority, false),
        AccountMeta::new(owner_token_account, false),
        AccountMeta::new_readonly(dataset_account, false),
        AccountMeta::new_readonly(token_program, false),
    ];

    let data = DataNexusInstruction::PurchaseAccess { hash, amount }.pack();

    Ok(Instruction {
        program_id,
        accounts,
        data,
    })
}

/// Creates a `ShareAccess` instruction
pub fn share_access(
    program_id: Pubkey,
    user_authority: Pubkey,
    user_access_account: Pubkey,
    recipient_authority: Pubkey,
    recipient_access_index: Pubkey,
    recipient_access_account: Pubkey,
    dataset_account: Pubkey,
    hash: [u8; 32],
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(user_authority, true),
        AccountMeta::new(user_access_account, false),
        AccountMeta::new(recipient_authority, false),
        AccountMeta::new(recipient_access_index, false),
        AccountMeta::new(recipient_access_account, false),
        AccountMeta::new_readonly(dataset_account, false),
    ];

    let data = DataNexusInstruction { hash }.pack();

    Ok(Instruction {
        program_id,
        accounts,
        data,
    })
}
