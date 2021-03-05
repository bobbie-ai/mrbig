#!/bin/bash

set -eux -o pipefail

# Build the testing binaries
cargo build --quiet \
      --bin test_grpc_service \
      --bin test_grpc_no_reflection \
      --bin test_grpc_multiple \
      --bin test_grpc_no_health \
      --bin test_grpc_traceable

# Project path
PROJECT_DIR=$(realpath $(pwd)/../..)
# Target path with build objects
TARGET_DIR=${PROJECT_DIR}/target/debug
# Path to coverage folder
COV_DIR=${PROJECT_DIR}/target/coverage
# Coverage command
COV="kcov --exclude-pattern=${TARGET_DIR} --include-pattern=${PROJECT_DIR} ${COV_DIR}"

########
echo "Test grpc services"
########
$COV ${TARGET_DIR}/test_grpc_service
$COV ${TARGET_DIR}/test_grpc_no_reflection
$COV ${TARGET_DIR}/test_grpc_multiple -d \
     | sed -r "s/\x1B\[([0-9]{1,3}(;[0-9]{1,2})?)?[mGK]//g" \
     | grep -E "DEBUG server{fakerequestid}: test_grpc_multiple: Greeter/SayHello (.*) -- OK"
$COV ${TARGET_DIR}/test_grpc_no_health
# Remove colors before testing output
$COV ${TARGET_DIR}/test_grpc_traceable -d \
     | sed -r "s/\x1B\[([0-9]{1,3}(;[0-9]{1,2})?)?[mGK]//g" \
     | grep "INFO server{fakerequestid}: test_grpc_traceable:" -A4 \
     | grep "INFO server{fakebadrequestid}: test_grpc_traceable:" -A1 \
     | grep -E "DEBUG server{fakebadrequestid}: test_grpc_traceable: Hotel/Rates (.*) -- ERR: status: NotFound, message:"
