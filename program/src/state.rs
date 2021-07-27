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
    pub shared_from: Pubkey,
    pub share_limit: u64,
}

impl Sealed for Access {}

impl IsInitialized for Access {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Access {
    const LEN: usize = 81;
    fn pack_into_slice(&self, dst: &mut [u8]) {
        dst[0] = self.is_initialized as u8;
        dst[1..33] = self.copy_from_slice(&self.hash);
        dst[33..65] = self.copy_from_slice(&self.key);
        dst[65..73] = self.copy_from_slice(&self.shared_from.to_bytes());
        dst[73..81] = self.copy_from_slice(&self.share_limit.to_le_bytes());
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let is_initialized = match src[0] {
            0 => false,
            1 => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        let hash: [u8; 32] = &src[1..33];
        let key: [u8; 32] = &src[33..65];
        let shared_from = Pubkey::from_array(&src[65..73]);
        let share_limit = u64::from_le_bytes(&src[73..81]);

        Ok(Self {
            is_initialized,
            hash,
            key,
            shared_from,
            share_limit,
        })
    }
}

pub struct Owned {
    pub is_initialized: bool,
    pub hash: [u8; 32],
    pub key: [u8; 32],
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
    const LEN: usize = 81;
    fn pack_into_slice(&self, dst: &mut [u8]) {
        dst[0] = self.is_initialized as u8;
        dst[1..33].copy_from_slice(&self.hash);
        dst[33..65].copy_from_slice(&self.key);
        dst[65..73].copy_from_slice(&self.value.to_le_bytes());
        dst[73..81].copy_from_slice(&self.share_limit.to_le_bytes());
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let is_initialized = match src[0] {
            0 => false,
            1 => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        let hash: [u8; 32] = &src[1..33];
        let key: [u8; 32] = &src[33..65];
        let value = u64::from_le_bytes(&src[65..73]);
        let share_limit = u64::from_le_bytes(&src[73..81]);

        Ok(Self {
            is_initialized,
            hash,
            key,
            value,
            share_limit,
        })
    }
}

pub struct UserState {
    pub is_initialized: bool,
    pub access: [Option<Access>; 128],
    pub owned: [Option<Owned>; 128],
    pub pointer: Option<[u8; 32]>,
}

impl IsInitialized for UserState {
    fn is_initialzed(&self) -> bool {
        self.is_initialized
    }
}

impl Sealed for UserState {}

impl Pack for UserState {
    const LEN: usize = 20769;
    const OWNED_BOUND: usize = 20737;
    const ACCESS_BOUND: usize = 10369;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        dst[0] = self.is_initialized as u8;
        dst[1..UserState::ACCESS_BOUND].copy_from_slice(&self.access);
        dst[UserState::ACCESS_BOUND..UserState::OWNED_BOUND].copy_from_slice(&self.owned);
        dst[UserState::OWNED_BOUND..UserState::LEN].copy_from_slice(&self.pointer);
    }
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let is_initialized = match src[0] {
            0 => false,
            1 => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        let access = {
            let mut offset: usize = 1;
            let mut access_vec = Vec::with_capacity(128);
            while offset != UserState::ACCESS_BOUND {
                let (data, src) = src[offset..UserState::ACCESS_BOUND].split_at(Access::LEN);
                let access = match Access::unpack_from_slice(data)? {
                    Access => Some(Access),
                    _ => None,
                };
                access_vec.push(access);
                offset += Access::LEN;
            }
            access_vec.collect()
        };

        let owned = {
            let mut offset: usize = UserState::ACCESS_BOUND;
            let mut owned_vec = Vec::with_capacity(128);
            while offset != UserState::OWNED_BOUND {
                let (data, src) = src[offset..UserState::OWNED_BOUND].split_at(Owned::LEN);
                let owned = match Owned::unpack_from_slice(data)? {
                    Owned => Some(Owned),
                    _ => None,
                };
                owned_vec.push(owned);
                offset += Owned::LEN;
            }
            owned_vec.collect()
        };

        let pointer = Pubkey::new_from_array(&src[UserState::OWNED_BOUND..UserState::LEN]);

        Ok(Self {
            is_initialized,
            access,
            owned,
            pointer,
        })
    }
}
