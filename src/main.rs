use {
    clap::{Arg, ArgMatches},
    inc_20210825::{audit, cleanup},
    solana_clap_utils::{
        input_validators::{
            is_url_or_moniker, is_valid_pubkey, is_valid_signer, normalize_to_url_if_moniker,
        },
        keypair::{signer_from_path, signer_from_path_with_config, SignerFromPathConfig},
    },
    solana_client::rpc_client::RpcClient,
    solana_remote_wallet::remote_wallet::RemoteWalletManager,
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signer},
    std::{process::exit, sync::Arc},
};

pub fn owner_keypair_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("owner")
        .value_name("OWNER_KEYPAIR")
        .validator(is_valid_signer)
        .required(true)
        .multiple(true)
        .help("Keypair or address of the token's owner")
}

pub fn mint_address_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("mint")
        .long("mint")
        .takes_value(true)
        .value_name("MINT_ADDRESS")
        .multiple(true)
        .number_of_values(1)
        .validator(is_valid_pubkey)
        .help("Address of the SPL token mint")
}

fn get_signer(
    matches: &ArgMatches<'_>,
    keypair_path: &str,
    wallet_manager: &mut Option<Arc<RemoteWalletManager>>,
    allow_null_signer: bool,
) -> Box<dyn Signer> {
    let config = SignerFromPathConfig { allow_null_signer };
    signer_from_path_with_config(matches, keypair_path, "owner", wallet_manager, &config)
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            exit(1);
        })
}

fn get_owners_and_mints(
    sub_matches: &ArgMatches<'_>,
    allow_null_signer: bool,
    wallet_manager: &mut Option<Arc<RemoteWalletManager>>,
) -> (Vec<Box<dyn Signer>>, Option<Vec<Pubkey>>) {
    let mints = if sub_matches.is_present("mint") {
        Some(
            sub_matches
                .values_of("mint")
                .unwrap()
                .map(|p| {
                    get_signer(
                        sub_matches,
                        p,
                        wallet_manager,
                        /* allow_null_signer = */ true,
                    )
                    .pubkey()
                })
                .collect::<Vec<_>>(),
        )
    } else {
        None
    };

    let owners = sub_matches
        .values_of("owner")
        .unwrap()
        .map(|p| get_signer(sub_matches, p, wallet_manager, allow_null_signer))
        .collect::<Vec<_>>();
    (owners, mints)
}

fn main() {
    let matches = clap::App::new("inc-20210805")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .arg({
            let arg = Arg::with_name("config_file")
                .short("C")
                .long("config")
                .value_name("PATH")
                .takes_value(true)
                .global(true)
                .help("Configuration file to use");
            if let Some(ref config_file) = *solana_cli_config::CONFIG_FILE {
                arg.default_value(config_file)
            } else {
                arg
            }
        })
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .takes_value(false)
                .global(true)
                .help("Show additional information"),
        )
        .arg(
            Arg::with_name("dry_run")
                .long("dry-run")
                .takes_value(false)
                .global(true)
                .help("Do all processing without sending transactions"),
        )
        .arg(
            Arg::with_name("json_rpc_url")
                .short("u")
                .long("url")
                .value_name("URL_OR_MONIKER")
                .takes_value(true)
                .global(true)
                .validator(is_url_or_moniker)
                .help(
                    "URL for Solana's JSON RPC or moniker (or their first letter): \
                       [mainnet-beta, testnet, devnet, localhost] \
                    Default from the configuration file.",
                ),
        )
        .subcommand(
            clap::SubCommand::with_name("audit")
                .about("Audit all accounts for the owners on the given mints")
                .arg(mint_address_arg())
                .arg(owner_keypair_arg()),
        )
        .subcommand(
            clap::SubCommand::with_name("cleanup")
                .about("Revoke all account delegations for the owners on the given mints")
                .arg(mint_address_arg())
                .arg(owner_keypair_arg()),
        )
        .get_matches();

    let mut wallet_manager = None;
    let config = {
        let cli_config = if let Some(config_file) = matches.value_of("config_file") {
            solana_cli_config::Config::load(config_file).unwrap_or_default()
        } else {
            solana_cli_config::Config::default()
        };
        let json_rpc_url = normalize_to_url_if_moniker(
            matches
                .value_of("json_rpc_url")
                .unwrap_or(&cli_config.json_rpc_url),
        );

        let fee_payer = signer_from_path(
            &matches,
            matches
                .value_of("fee_payer")
                .unwrap_or(&cli_config.keypair_path),
            "fee_payer",
            &mut wallet_manager,
        )
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            exit(1);
        });

        inc_20210825::config::Config {
            rpc_client: RpcClient::new_with_commitment(json_rpc_url, CommitmentConfig::confirmed()),
            fee_payer,
            dry_run: matches.is_present("dry_run"),
            verbose: matches.is_present("verbose"),
        }
    };

    match matches.subcommand() {
        ("audit", Some(sub_matches)) => {
            let (owners, mints) = get_owners_and_mints(sub_matches, true, &mut wallet_manager);
            audit::run(config, owners, mints);
        }
        ("cleanup", Some(sub_matches)) => {
            let (owners, mints) = get_owners_and_mints(sub_matches, false, &mut wallet_manager);
            cleanup::run(config, owners, mints);
        }
        _ => unreachable!(),
    }
}
