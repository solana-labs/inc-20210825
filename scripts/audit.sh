#!/usr/bin/env bash

set -e

MINT=$1
TARGET_OWNER=$2

COMMON=${COMMON:-"--verbose -ul"}
INC_20210825=${INC_20210825:-./target/release/inc-20210825}

$INC_20210825 $COMMON audit --mint $MINT $TARGET_OWNER
