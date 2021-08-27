## Usage
### Install prerequisites
#### System development libraries
```
sudo apt install libssl-dev libudev-dev
```
#### Rust
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
### Targeted vulnerable account cleanup
The following command will revoke any existing delegations on all wallet:mint
pairs provided. Specify addresses for all mints that your organization supports
as well as every user deposit SOL wallet generated before epoch 216. Wallets
must be specified as the path to a keypair file in `solana-keygen new` format.
It may be necessary to run this command in multiple batches if the command line
is too long for the shell.
#### Dry-run
First a dry-run to be sure everything looks OK
```
cargo run -- cleanup \
--dry-run \
--mint MINT1_ADDRESS \
--mint MINT2_ADDRESS \
... \
--mint MINTN_ADDRESS \
DEPOSIT_SOL_WALLET1_PATH \
DEPOSIT_SOL_WALLET2_PATH \
... \
DEPOSIT_SOL_WALLETN_PATH
```
#### Effective run
If everything looks OK from the [dry-run](#dry-run), run the same command again
with the `--dry-run` argument removed.
### Targeted transaction history audit
The following will generate an audit report for the transaction history of each
token account, flagging suspicious and malicious behavior. As with
[cleanup](#targeted-vulnerable-account-cleanup), specify the addresses for every
mint your organization supports as well as every user deposit SOL wallet
generated before epoch 216. Wallets must be specified as the path to a keypair
file in `solana-keygen new` format. It may be necessary to run this command in
multiple batches if the command line is to long for the shell.
#### Run
```
cargo run -- audit \
--mint MINT1_ADDRESS \
--mint MINT2_ADDRESS \
... \
--mint MINTN_ADDRESS \
DEPOSIT_SOL_WALLET1_PATH \
DEPOSIT_SOL_WALLET2_PATH \
... \
DEPOSIT_SOL_WALLETN_PATH | tee report.csv
```
### Full vulnerable account cleanup
It is possible that an attacker created vulnerable accounts for mints that your
organization does not yet support in the hope that one day they will be supported
and deposits can be exploited. To clean up all potentially vulnerable accounts,
re-run the [cleanup](#targeted-vulnerable-account-cleanup) command, this time
omitting all `--mint ...` arguments. This process may take quite some time depending
on how many unique tokens have been sent to each wallet.
