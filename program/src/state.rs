use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use crate::error::DataNexusError::InvalidInstruction;

use std::{convert::TryInto, mem::size_of};

pub struct OwnerState {
    pub is_initialized: bool,
    pub pointer: Option<Pubkey>,
    pub datasets: [Option<Pubkey>; 128],
}

impl IsInitialized for OwnerState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Sealed for OwnerState {}

impl Pack for OwnerState {
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

pub struct AccessState {
    pub is_initialized: bool,
    pub pointer: Option<Pubkey>,
    pub datasets: [AccessInfo; 128],
}

pub struct AccessInfo {
    pub hash: [u8; 32],
    pub key: [u8; 32],
    pub shared_from: Pubkey,
    pub share_limit: u64,
}

impl IsInitialized for AccessHeader {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Sealed for AccessState {}

impl Pack for AccessState {
    const LEN: usize = 33 + (128 * 80);
    fn pack_into_slice(&self, dst: &mut [u8]) {
        dst[0] = self.is_initialized as u8;
        dst[1..33].copy_from_slice(self.pointer.as_ref());
        dst[33..Self::LEN].copy_from_slice({
            let dataset_vec = Vec::with_capacity(Self::LEN - 33);
            for dataset in self.datasets {
                dataset_vec.push(dataset.pack())
            }
            dataset_vec[..]
        })
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let is_initialized = match src[0] {
            0 => false,
            1 => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        let pointer = Pubkey::new_from_array(src[1..33].try_into().unwrap());

        let datasets = AccessInfo::unpack(src[33..Self::LEN])?;

        Ok(Self {
            is_initialized,
            pointer,
            datasets,
        })
    }
}

impl AccessInfo {
    fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        buf.extend_from_slice(&self.hash);
        buf.extend_from_slice(&self.key);
        buf.extend_from_slice(self.shared_from.as_ref());
        buf.extend_from_slice(&self.share_limit.to_le_bytes());

        buf
    }

    fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        let hash: [u8; 32] = data[..32].try_into().unwrap();
        let key: [u8; 32] = src[32..64].try_into().unwrap();
        let shared_from = Pubkey::new_from_array(&src[64..72].try_into().unwrap());
        let share_limit = u64::from_le_bytes(&src[72..80].try_into().unwrap());

        Ok(Self {
            hash,
            key,
            shared_from,
            share_limit,
        })
    }
}

pub struct DatasetState {
    pub is_initialized: bool,
    pub owner: Pubkey,
    pub hash: [u8; 32],
    pub key: Option<[u8; 32]>,
    pub value: Option<u64>,
    pub share_limit: Option<u16>,
}

impl IsInitialized for DatasetState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Sealed for DatasetState {}

impl Pack for DatasetState {
    const LEN: usize = 107;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        dst[0] = self.is_initializes as u8;
        dst[1..33].copy_from_slice(self.owner.as_ref());
        dst[33..65].copy_from_slice(&self.hash);
        dst[65..97].copy_from_slice(&self.key);
        dst[97..105].copy_from_slice(&self.value.to_le_bytes());
        dst[105..107].copy_from_slice(&self.share_limit.to_le_bytes());
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let is_initialized = match src[0] {
            0 => false,
            1 => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        let owner = Pubkey::new_from_array(&src[1..33].try_into().unwrap());
        let hash = src[33..65].try_into().unwrap();
        let key = src[65..97].try_into().unwrap();
        let value = u64::from_le_bytes(&src[97..105].try_into().unwrap());
        let share_limit = u16::from_le_bytes(&src[105..107].try_into().unwrap());

        Ok(Self {
            is_initialized,
            owner,
            hash,
            key,
            value,
            share_limit,
        })
    }
}
