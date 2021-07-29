use solana_sdk::{
    self, hash::hashv, msg, program_pack::Pack, pubkey::Pubkey, signature::Signer, system_program,
    transaction::Transaction,
};

use solana_clap_utils::{
    input_parsers::{pubkey_of, value_of},
    input_validators::{is_amount, is_hash, is_keypair, is_parsable, is_pubkey, is_url},
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
        AccountType, Params,
    },
    state::AccountState,
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
    let dataset_state = AccountState::unpack_from_slice(&dataset_buf)?;
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

fn command_share_access(config: &Config, recipient_authority: Pubkey, hash: [u8; 32]) {
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

fn main() {
    let app_matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg({
            let arg = Arg::with_name("config_file")
                .short("c")
                .long("config")
                .value_name("PATH")
                .takes_value(true)
                .global(true)
                .help("Configuration file to use");
            if let Some(ref config_file) = *solana_cli_config::CONFIG_FILE {
                arg.default_value(&config_file)
            } else {
                arg
            }
        })
        .arg(
            Arg::with_name("rpc_url")
                .short("u")
                .long("url")
                .validator(is_url)
                .takes_value(true)
                .global(true)
                .help("Specifiy target Solana cluster"),
        )
        .arg(
            Arg::with_name("payer")
                .short("p")
                .long("payer")
                .value_name("KEYPAIR")
                .validator(is_keypair)
                .takes_value(true)
                .help(
                    "Specify payer. \
                Defaults to client keypair.",
                ),
        )
        .subcommand(
            Subcommand::with_name("create")
                .about("Create an account")
                .arg(
                    Arg::with_name("account_type")
                        .short("t")
                        .long("type")
                        .value_name("ACCOUNT TYPE")
                        .takes_value(true)
                        .required(true)
                        .index(1)
                        .help(
                            "Specify a user account type to create: \
                    owner \
                    access \
                    data",
                        ),
                )
                .arg(
                    Arg::with_name("authority")
                        .short("a")
                        .long("authority")
                        .value_name("PUBKEY")
                        .validator(is_pubkey)
                        .takes_value(true)
                        .help("Public Key of the owner of the created account"),
                )
                .arg(
                    Arg::with_name("hash")
                        .short("h")
                        .long("hash")
                        .validator(is_hash)
                        .value_name("HASH")
                        .takes_value(true)
                        .help("Hash for dataset accounts"),
                ),
        )
        .subcommand(
            Subcommand::with_name("set")
                .about("Set dataset parameters")
                .arg(
                    Arg::with_name("hash")
                        .short("h")
                        .long("hash")
                        .value_name("HASH")
                        .validator(is_hash)
                        .takes_value(true)
                        .required(true)
                        .index(1)
                        .help("Hash of target dataset"),
                )
                .arg(
                    Arg::with_name("key")
                        .short("k")
                        .long("key")
                        .value_name("KEY")
                        .takes_value(true)
                        .help("Key for target dataset"),
                )
                .arg(
                    Arg::with_name("value")
                        .short("v")
                        .long("value")
                        .value_name("AMOUNT")
                        .validator(is_amount)
                        .takes_value(true)
                        .help("Value of target dataset in USD"),
                )
                .arg(
                    Arg::with_name("share_limit")
                        .short("l")
                        .long("share-limit")
                        .value_name("AMOUNT")
                        .validator(is_amount)
                        .takes_value(true)
                        .help("Number of times the target dataset can be shared"),
                )
                .arg(
                    Arg::with_name("reference_data")
                        .short("r")
                        .long("ref-data")
                        .value_name("PUBKEY")
                        .validator(is_pubkey)
                        .takes_value(true)
                        .help("Address of the dataset the target dataset is derived from"),
                ),
        )
        .subcommand(
            Subcommand::with_name("purchase_access")
                .about("Purchase access to a dataset")
                .arg(
                    Arg::with_name("hash")
                        .short("h")
                        .long("hash")
                        .value_name("HASH")
                        .validator(is_hash)
                        .takes_value(true)
                        .required(true)
                        .index(1)
                        .help("Hash of target dataset"),
                )
                .arg(
                    Arg::with_name("value")
                        .short("v")
                        .long("value")
                        .value_name("AMOUNT")
                        .validator(is_amount)
                        .takes_value(true)
                        .is_required(true)
                        .index(2)
                        .help("Value of dataset to purchase access to"),
                ),
        )
        .subcommand(
            Subcommand::with_name("share_access")
                .about("Share access to a dataset")
                .arg(
                    Arg::with_name("hash")
                        .short("h")
                        .long("hash")
                        .value_name("HASH")
                        .validator(is_hash)
                        .takes_value(true)
                        .required(true)
                        .index(1)
                        .help("Hash of the target dataset"),
                )
                .arg(
                    Arg::with_name("recipient")
                        .short("r")
                        .long("recipient")
                        .value_name("PUBKEY")
                        .validator(is_pubkey)
                        .takes_value(true)
                        .required(true)
                        .index(2)
                        .help("Address to share access to"),
                ),
        )
        .get_matches();

    let mut wallet_manager = None;
    let (sub_command, sub_matches) = app_matches.subcommand();
    let matches = sub_matches.unwrap();

    // Retrieves payer keypair and target RPC from the config file
    let config = {
        let cli_config = if let Some(config_file) = matches.value_of("config file") {
            solana_cli_config::Config::load(config_file).unwrap_or_defaul()
        } else {
            solana_cli_config::Config::default()
        };

        let rpc_url = &cli_config.json_rpc_url;
        let default_signer_arg_name = "owner".to_string();
        let default_signer_path = cli_config.keypair_path.clone();
        let default_signer = DefaultSigner::new(default_signer_arg_name, default_signer_path);

        let payer = {
            let config = SignerFromPathConfig {
                allow_null_signer: true,
            };
            let payer = default_signer
                .signer_from_path_with_config(&matches, &mut wallet_manager, &config)
                .unwrap_or_else(|e| {
                    eprintln!("error: {}", e);
                    std::process::exit(1);
                });
            payer
        };

        Config {
            rpc_client: RpcClient::new(rpc_url.clone()),
            payer,
        }
    };

    match (sub_command, sub_matches) {
        ("create", Some(args)) => {
            let authority = if let Some(pubkey) = value_of(args, "authority") {
                pubkey
            } else {
                config.payer.pubkey()
            };
            match value_of(args, "account_type").unwrap() {
                "owner" => command_init_user_account(config, authority, AccountType::Owner),
                "access" => command_init_user_account(config, authority, AccountType::Access),
                _ => command_init_data_account(config, hash),
            };
        }
        ("set", Some(args)) => {
            let hash = value_of(args, "hash").unwrap();
            let key = if let Some(key) = value_of(args, "key") {
                key
            } else {
                None
            };
            let value = if let Some(value) = value_of(args, "value") {
                value
            } else {
                None
            };
            let share_limit = if let Some(share_limit) = value_of(args, "share_limit") {
                share_limit
            } else {
                None
            };
            let ref_data = if let Some(ref_data) = value_of(args, "reference_data") {
                ref_data
            } else {
                None
            };

            let params = match (key, value, share_limit, ref_data) {
                (key, None, None, None) => Params::Key(key),
                (None, value, None, None) => Params::Value(value),
                (None, None, share_limit, None) => Params::ShareLimit(share_limit),
                (None, None, None, ref_data) => Params::ReferenceData(ref_data),
                _ => Params::Init(key, value, share_limit, ref_data),
            };

            command_set_data_params(config, hash, params);
        }
        ("purchse_access", Some(args)) => {
            let hash = value_of(args, "hash").unwrap();
            let value = value_of(args, "amount").unwrap();
            command_purchase_access(config, hash, value);
        }
        ("share_access", Some(args)) => {
            let hash = value_of(args, "hash").unwrap();
            let recipient = pubkey_of(args, "recipient").unwrap();
            command_share_access(config, recipient, hash);
        }
        _ => unreachable!(),
    };
}
