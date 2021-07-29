use datanexus::datanexus_program;
use solana_sdk::pubkey::Pubkey;

const OWNER_MARKER: &[u8; 5] = b"owner";
const ACCESS_MARKER: &[u8; 6] = b"access";

pub fn create_owner_address(authority: Pubkey) -> Pubkey {
    Pubkey::create_program_address(
        &[OWNER_MARKER, authority.as_ref()],
        datanexus_program.as_ref(),
    )
}

pub fn get_owner_address(authority: Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[OWNER_MARKER, authority.as_ref()],
        datanexus_program.as_ref(),
    )
    .0
}

pub fn create_access_address(authority: Pubkey) -> Pubkey {
    Pubkey::create_program_address(
        &[ACCESS_MARKER, authority.as_ref()],
        datanexus_program.as_ref(),
    )
}

pub fn get_access_address(authority: Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[ACCESS_MARKER, authority.as_ref()],
        datanexus_program.as_ref(),
    )
    .0
}

pub fn create_dataset_address(hash: &[u8; 32]) -> Pubkey {
    Pubkey::create_program_address(&[hash], datanexus_program.as_ref())
}

pub fn get_dataset_address(hash: &[u8; 32]) -> Pubkey {
    Pubkey::find_program_address(&[hash], datanexus_program.as_ref()).0
}

pub fn create_associated_access_address(authority: Pubkey, dataset_address: Pubkey) -> Pubkey {
    Pubkey::create_program_address(
        &[authority.as_ref(), dataset_address.as_ref()],
        datanexus_program.as_ref(),
    )
}

pub fn get_associated_access_address(authority: Pubkey, dataset_address: Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[authority.as_ref(), dataset_address.as_ref()],
        datanexus_program.as_ref(),
    )
    .0
}
