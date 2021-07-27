use solana_program::{
    instruction::{AccountMeta, Instruction},
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::error::DataNexusError;

enum AccountType {
    Owner,
    Access,
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
    /// `[w,s]` Payer Account
    /// `[w]` Authority
    /// `[w]` Owner/Access Account
    /// `[w]` System Program
    InitUserAccount(Account),

    /// Initialize Dataset Account
    ///
    /// Accounts expected:
    ///
    /// `[w,s]` Authority
    /// `[w]` Owner Account
    /// `[w]` Dataset Account
    /// `[w]` System Program
    InitDataAccount { hash: [u8; 32] },

    /// Write to Buffer
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
    /// `[w]` User Access Account
    /// `[w]` Recipient Authority
    /// `[w]` Recipient Access Account
    /// `[]` Dataset Account
    ShareAccess { hash: [u8; 32] },
}

impl AdminInstruction {
    pub fn pack(&self) -> Vec<u8> {}
    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {}
}

/// Creates an `InitUserAccount` instruction
pub fn init_owner_account(
    program_id: Pubkey,
    payer: Pubkey,
    authority: Pubkey,
    owner_account: Pubkey,
    system_program: Pubkey,
    account_type: AccountType,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(payer, true),
        AccountMeta::new(authority, false),
        AccountMeta::new(owner_account, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let data = DataNexusInstruction::InitUserAccount(account_type).pack();

    Ok(Instruction {
        program_id,
        accounts,
        data,
    })
}

/// Creates an `InitDataAccount` instruction
pub fn init_data_account(
    program_id: Pubkey,
    authority: Pubkey,
    owner_account: Pubkey,
    dataset_account: Pubkey,
    system_program: Pubkey,
    hash: [u8; 32],
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(authority, true),
        AccountMeta::new(owner_account, false),
        AccountMeta::new(dataset_account, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let data = DataNexusInstruction::InitDataAccount { hash }.pack();

    Ok(Instruction {
        program_id,
        accounts,
        data,
    })
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
        AccountMeta::new(user_token_account, false),
        AccountMeta::new(owner_authority, false),
        AccountMeta::new(owner_token_account, false),
        AccountMeta::new_readonly(dataset_account, false),
        AccountMeta::new_readonly(token_program, false),
    ];

    let data = DataNexusInstruction::PurchaseAccess { hash, amount }.pack();

    Ok(Self {
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
    recipient_access_account: Pubkey,
    dataset_account: Pubkey,
    hash: [u8; 32],
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(user_authority, true),
        AccountMeta::new(user_access_account, false),
        AccountMeta::new(recipient_authority, false),
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
