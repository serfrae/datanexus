use solana_sdk::{
    self, hash::hashv, msg, program_pack::Pack, pubkey::Pubkey, signature::Signer, system_program,
    transaction::Transaction,
};

use solana_clap_utils::{
    input_parsers::{pubkey_of, value_of},
    input_validators::{is_amount, is_keypair, is_parsable, is_pubkey, is_url},
    keypair::DefaultSigner,
};

use solana_client::rpc_client::RpcClient;

use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};
use spl_token::state::Account;

use clap::{crate_description, crate_name, crate_version, App, AppSettings, Arg, SubCommand};

use datanexus::{
    datanexus_program,
    instruction::{
        init_data_account, init_user_account, purchase_access, set_data_params, share_access,
        AccountType,
    },
    state::DatasetState,
};

use datanexus_utils::*;

struct Config {
    payer: Box<dyn Signer>,
    rpc_client: RpcClient,
}

fn sign_and_send_transaction(config: &Config, instructions: [Instruction]) -> Signature {
    let mut transaction = Transaction::new_with_payer(&instructions, Some(config.payer.pubkey()));
    let recent_blockhash = config.rpc_client.get_recent_blockhash().unwrap().0;
    transaction.sign([&*config.payer], recent_blockhash);

    config
        .rpc_client
        .send_and_confirm_transaction_with_spinner(&transaction)
        .unwrap();
}

fn command_init_user_account(config: &Config, authority: Pubkey, account_type: AccountType) {
    let user_account = if account_type == AccountType::Owner {
        create_owner_address(authority)
    } else {
        create_access_address(authority)
    };

    let instructions = [init_user_account(
        datanexus_program,
        config.payer.pubkey(),
        authority,
        user_account,
        system_program,
        account_type,
    )?];

    let signature = sign_and_send_transaction(config, instructions);

    println!("User {} Account Created: {}", account_type, user_account);
    println!("Transaction Signature: {}", signature);
}

fn command_init_data_account(config: &Config, hash: [u8; 32]) {
    let owner_account = get_owner_account(config.payer.pubkey().as_ref());
    let dataset_account = create_associated_dataset_address(&hash);

    let instructions = [init_data_account(
        datanexus_program,
        config.payer.pubkey(),
        owner_account,
        dataset_account,
        system_program,
        hash,
    )?];

    let signature = send_and_sign_transaction(config, instructions);

    println!("Dataset Account for {} Created: {}", hash, dataset_account);
    println!("Transaction Signature: {}", signature);
}

fn command_set_data_params(config: &Config, hash: [u8; 32], params: Params) {
    let dataset_account = get_associated_dataset_address(&hash);
    let instructions = [set_data_params(
        datanexus_program,
        config.payer.pubkey(),
        dataset_account,
        hash,
        params,
    )?];

    let signature = sign_and_send_transaction(config, [dataset_account], instructions);
    println!("Transaction Signature: {}", signature);
}

fn command_purchase_access(
    config: &Config,
    hash: [u8; 32],
    user_token_account: Pubkey,
    amount: u64,
) {
    let dataset_address = get_dataset_address(&hash);
    let user_access_account = get_access_address(config.payer.pubkey().as_ref());
    let user_associated_access_account =
        create_associated_access_address(config.payer.pubkey().as_ref(), dataset_address.as_ref());

    let user_token_account_buf = config
        .rpc_client
        .get_account_data(&user_token_account)
        .unwrap();
    let user_token_account_state = Account::unpack_from_slice(user_token_account_buf)?;
    let token_mint = user_token_account_state.mint;
    let dataset_buf = config
        .rpc_client
        .get_account_data(&dataset_account)
        .unwrap()
        .clone();
    let dataset_state = DatasetState::unpack_from_slice(&dataset_buf)?;
    let owner_token_account = get_associated_token_address(&dataset_state.owner, &token_mint);

    let instructions = [purchase_access(
        datanexus_program,
        config.payer.pubkey(),
        user_access_account,
        user_associated_access_account,
        owner_authority,
        owner_token_account,
        dataset_address,
        spl_token::ID,
        hash,
        amount,
    )?];

    let signature = sign_and_send_transaction(config, instructions);

    println!("purchase test");
}

fn command_share_access(
    config: &Config,
    user_authority: Pubkey,
    recipient_authority: Pubkey,
    hash: [u8; 32],
) {
    let dataset_address = get_dataset_address(&hash);
    let user_associated_access_account =
        get_associated_access_address(config.payer.pubkey().as_ref(), dataset_address.as_ref());
    let recipient_access_account = get_access_address(recipient_authority.as_ref());
    let recipient_associated_access_account =
        create_associated_access_address(recipient_authority.as_ref(), dataset_address.as_ref());

    let instructions = [share_access(
        datanexus_program,
        config.payer.pubkey(),
        user_associated_access_account,
        repicient_authority,
        recipient_access_account,
        recipient_associated_access_account,
        hash,
    )?];

    let signature = send_and_sign_transaction(config, instructions);

    println!("share acess test");
}

fn main() {}
