use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use crate::error::DataNexusError::InvalidInstruction;

use std::convert::TryInto;

pub enum AccountFlag {
    Access,
    Dataset,
}

pub struct AccountIndex {
    pub is_initialized: bool,
    pub pointer: Option<Pubkey>,
    pub datasets: [Option<Pubkey>; 128],
}

impl IsInitialized for AccountIndex {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Sealed for AccountIndex {}

impl Pack for AccountIndex {
    const LEN: usize = 33 + 4096;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        dst[0] = self.is_initialized as u8;
        dst[1..33].copy_from_slice(match self.pointer {
            Some(n) => n.to_bytes(),
            None => [0u8; 32],
        });
        dst[33..Self::LEN].copy_from_slice({
            let dataset_vec = Vec::with_capacity(Self::LEN - 33);
            for dataset in self.datasets {
                dataset_vec.push(match dataset {
                    Some(n) => n.to_bytes(),
                    None => [0u8; 32],
                })
            }
            dataset_vec
        });
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let is_initialized = match src[0] {
            0 => false,
            1 => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        let pointer = match src[1..33].try_into().unwrap() {
            x if x = [0u8; 32] => None,
            _ => Some(Pubkey::new_from_array(src[1..33].try_into().unwrap())),
        };
        let datasets = {
            let dataset_vec = Vec::with_capacity(Self::LEN - 33);
            let mut offset = 33;
            while offset < Self::LEN {
                dataset_vec.push(match src[offset..offset + 32].try_into().ok() {
                    x if x = [0u8; 32] => None,
                    _ => Some(Pubkey::new_from_array(
                        src[offset..offset + 32].try_into().unwrap(),
                    )),
                });
                offset += 32;
            }
            dataset_vec.collect()
        };

        Ok(Self {
            is_initialized,
            pointer,
            datasets,
        })
    }
}

pub struct AccountState {
    pub is_initialized: bool,
    pub flag: AccountFlag,
    pub owner: Pubkey,
    pub hash: [u8; 32],
    pub key: Option<[u8; 32]>,
    pub value: Option<u64>,
    pub share_limit: Option<u16>,
}

impl IsInitialized for AccountState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Sealed for AccountState {}

impl Pack for AccountState {
    const LEN: usize = 108;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        dst[0] = self.is_initialized as u8;
        dst[1] = match self.flag {
            AccountFlag::Access => 0u8,
            AccountFlag::Dataset => 1u8,
        };
        dst[2..34].copy_from_slice(self.owner.as_ref());
        dst[34..66].copy_from_slice(&self.hash);
        dst[66..98].copy_from_slice(&self.key);
        dst[98..106].copy_from_slice(&self.value.to_le_bytes());
        dst[106..108].copy_from_slice(&self.share_limit.to_le_bytes());
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let is_initialized = match src[0] {
            0 => false,
            1 => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        let flag = match [1] {
            0 => AccountFlag::Access,
            1 => AccountFlag::Dataset,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        let owner = Pubkey::new_from_array(&src[2..34].try_into().unwrap());
        let hash = src[34..66].try_into().unwrap();
        let key = src[66..98].try_into().unwrap();
        let value = u64::from_le_bytes(&src[98..106].try_into().unwrap());
        let share_limit = u16::from_le_bytes(&src[106..108].try_into().unwrap());

        Ok(Self {
            is_initialized,
            flag,
            owner,
            hash,
            key,
            value,
            share_limit,
        })
    }
}
