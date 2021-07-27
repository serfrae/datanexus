use solana_program::{
    instruction::{AccountMeta, Instruction},
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::error::DataNexusError;

pub enum AdminInstruction {
    /// Initialize Dataset Account
    ///
    /// Accounts expected:
    ///
    /// `[w]` Dataset Account
    /// `[w,s]` Authority
    /// `[w]` System Program
    Init { hash: [u8; 32] },

    /// Write to Buffer
    ///
    /// Accounts expected:
    ///
    /// `[w]` Dataset Account
    /// `[w,s]` Authority
    Write { hash: [u8; 32] },

    /// Set Dataset Price
    ///
    /// Accounts expected:
    ///
    /// `[w]` Dataset Account
    /// `[w,s]` Authority
    Settings {
        hash: [u8; 32],
        price: u64,
        share_limit: u64,
        reference_data: [u8; 32],
    },

    /// Close Account
    ///
    /// Accounts expected:
    ///
    /// `[w]` Dataset Account
    /// `[w,s]` Authority
    Close { hash: [u8; 32] },
}

impl AdminInstruction {
    pub fn pack(&self) -> Vec<u8> {}
    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {}
}

pub enum UserInstruction {
    /// Initialize User Account
    ///
    /// Accounts expected:
    ///
    /// `[w]` User Account
    /// `[w,s]` User Authority
    /// `[]` System Program
    Init,

    /// Purchase Dataset Access
    ///
    /// Accounts expected:
    ///
    /// `[w]` User Account
    /// `[w,s]` User Authority
    /// `[w]` Admin Wallet
    /// `[]` Dataset Account
    /// `[]` Serum DEX
    /// `[]` Token Program
    Buy { hash: [u8; 32], amount: u64 },

    /// Share Dataset Access
    Share { hash: [u8; 32] },
    // Maybe a convenience function to allow for reading the buffer data?
}

impl UserInstruction {
    pub fn pack(&self) -> Vec<u8> {}
    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {}
}
