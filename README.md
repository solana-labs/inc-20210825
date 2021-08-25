# TODO
## `audit` subcommand
### Inputs
  * One or more mint address (via `--mint`)
  * One or more SOL wallet addresses (position args)
### Outputs
  * JSON formatted report with schema as follows
  ```
  [
    {
      "walletAddress": bs58_address_string,
      "tokenAddresses": [
        {
          "tokenAddress": bs58_address_string,
          "mintAddress": bs58_address_string,
          "delegateAddress": null / bs58_address_string,
          "delegateTransfers": [
            {
              "transacionID": bs58_signature_string,
              "signer": bs58_pubkey_string,
              "amount": token_ui_amount_string,
            }
            ,
            ...
          ],
          "ownerChanges": [
            {
              "transactionId": bs58_signature_string,
              "signer": bs58_pubkey_string,
              "new_owner: bs58_pubkey_string,
            },
            ...
          ],
          "delegateChanges": [
            {
              "transactionId": bs58_signature_string,
              "signer": bs58_pubkey_string,
              "new_delegate": null / bs58_pubkey_string,
            },
            ...
          ]
        },
        ...
      ]
    },
    ...
  ]
  ```
### Pseudo code
```
let report = new()
foreach wallet in wallets:
  let wallet_entry = new(wallet.address)
  foreach mint in mints:
    let accounts = get_token_accounts_for_owner(wallet, mint)
    foreach account in accounts:
      let token_entry = new(mint.address, account.address, account.delegate)
      let transactions = get_transaction_for_account(account)
      foreach transaction in transactions:
        if failed(transaction):
          continue
        let instructions = transaction_get_instructions(transaction)
        for instruction in instructions:
          if is_spl_token_transfer(instruction):
            if delegate_signed(instruction):
              let delegate_transfer_entry = new(transaction.id, instruction.signer, instruction.ui_amount)
              push(token_entry.delegate_tranfers, delegate_transfer_entry)
          elif is_spl_token_authorize_owner(instruction):
            let owner_change_entry = new(transaction.id, instruction.signer, instruction.new_delegate)
            push(token_entry.owner_changes, owner_change_entry)
          elif is_spl_token_approve(instruction):
            let delegate_change_entry = new(transaction.id, instruction.signer, instruction.new_delegate)
            push(token_entry.delegate_changes, delegate_change_entry)
      push(wallet_entry.token_address(token_entry))
  push(report, wallet_entry)
```

## `cleanup` subcommand
### Inputs
  * One or more mint address (via `--mint`)
  * One or more SOL wallet addresses (position args)
### Outputs
  * _None_
### Pseudo code
```
foreach wallet in wallets:
  foreach mint in mints:
    let accounts = get_token_accounts_for_owner(wallet, mint)
    foreach account in accounts:
      if !delegate_is_set(account):
        spl_token_revoke(account)
        print account.address
```
