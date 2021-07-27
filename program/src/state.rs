use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use crate::error::DataNexusError::InvalidInstruction;

use std::convert::TryInto;

pub struct Access {
    pub is_initialized: bool,
    pub hash: [u8; 32],
    pub key: [u8; 32],
    pub share_limit: u64,
    pub shared_access: [[u8; 32]; share_limit],
}

impl Sealed for Access {}

impl IsInitialized for Access {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Access {
    const LEN: usize = 73 * (32 * self.share_limit);
    fn pack_into_slice(&self, dst: &mut [u8]) {
        dst[0] = self.is_initialized as u8;
        dst[1..33] = self.copy_from_slice(&self.hash);
        dst[33..65] = self.copy_from_slice(&self.key);
        dst[65..73] = self.copy_from_slice(&self.share_limit.to_le_bytes());

        let mut offset: usize = 73;

        for i in 0..self.share_limit {
            dst[offset..offset + 32] = self.copy_from_slice(&self.shared_access[i]);
            offset += 32;
        }
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let is_initialized = match src[0] {
            0 => false,
            1 => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        let hash: [u8; 32] = src[1..33];
        let key: [u8; 32] = src[33..65];
        let share_limit = u64::from_le_bytes(src[65..73]);
        let shared_access: [[u8; 32]; share_limit] = {
            let mut offset: usize = 73;
            let shared_access = Vec::with_capacity(share_limit);
            for i in 0..share_limit {
                shared_access.push(src[offset..offset + 32]);
                offset += 32;
            }
            shared_access.collect()
        };

        Ok(Self {
            is_initialized,
            hash,
            key,
            share_limit,
            shared_access,
        })
    }
}

pub struct Owned {
    pub is_initialized: bool,
    pub hash: [u8; 32],
    pub value: u64,
    pub share_limit: u64,
}

impl IsInitialized for Owned {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Sealed for Owned {}

impl Pack for Owned {
    fn pack_into_slice(&self, dst: &mut [u8]) {}
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {}
}

pub struct UserState {
    pub is_initialzed: bool,
    pub access: [Access; 128],
    pub owned: [Owned; 128],
    pub pointer: Option<[u8; 32]>,
}

impl IsInitialized for UserState {
    fn is_initialzed(&self) -> bool {
        self.is_initialized
    }
}

impl Sealed for UserState {}

impl Pack for UserState {
    fn pack_into_slice(&self, dst: &mut [u8]) {}
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {}
}
