use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};

use num_derive::FromPrimitive;
use num_traits::FromPrimitive as FromPrimitiveTrait;

use thiserror::Error;

#[derive(Clone, Copy, Debug, Error, Eq, FromPrimitive, PartialEq)]
pub enum DataNexusError {
    #[error("Invalid Instruction")]
    InvalidInstruction,
}

impl From<DataNexusError> for ProgramError {
    fn from(e: DataNexusError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for DataNexusError {
    fn type_of() -> &'static str {
        "DataNexus Error"
    }
}

impl PrintProgramError for DataNexusError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitiveTrait,
    {
        match self {
            DataNexusError::InvalidInstruction => msg!("Invalid Instruction"),
        }
    }
}
