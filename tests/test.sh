#!/usr/bin/env bash
#
# Very basic set of tests until proper testing is implemented with rust
#

readonly LOG_LEVEL="${LOG_LEVEL:-WARNING}"
readonly CONFIG="$PWD/.novops.yml"

# if [ -z "$BW_SESSION" ]; then
# 	echo "Please export BW_SESSION to allow fetching from Bitwarden"
# 	exit 1
# fi

# Shortcut to use locally built novops
# Otherwise will work if available on PATH
export PATH=$PATH:$PWD/target/x86_64-unknown-linux-musl/release

test_start() {
	echo
	echo ">>> Start testing: $1"
}

test_basic() {
	local TEST_FOLDER
  TEST_FOLDER=$(mktemp -d)
	test_start "Basic tests"

	novops -e dev -c tests/.novops.yml -w "${TEST_FOLDER}"
	source "${TEST_FOLDER}/vars"

  env | grep NOVOPS
  ls -al "${TEST_FOLDER}"

	# Check variables are set according to config
	# See tests/.novops.yml
	[[ "$MY_APP_HOST" == "localhost" ]] || { echo "Expected var MY_APP_HOST == localhost, got $MY_APP_HOST"; return 1; }
	[[ "$NOVOPS_TEST_APP_FILE_CAT" == "/tmp/cat" ]] || { echo "Expected var NOVOPS_TEST_APP_FILE_CAT == /tmp/cat, got $NOVOPS_TEST_APP_FILE_CAT"; return 1; }
	[[ "$NOVOPS_TEST_APP_FILE_DOG" == "${TEST_FOLDER}/file_dog" ]] || { echo "Expected var NOVOPS_TEST_APP_FILE_DOG == ${TEST_FOLDER}/file_dog, got $NOVOPS_TEST_APP_FILE_DOG"; return 1; }
	[[ "$RAT_PATH_CUSTOM_NOVOPS_VAR" =~ ^${TEST_FOLDER}/file_* ]] || { echo "Expected var RAT_PATH_CUSTOM_NOVOPS_VAR =~ ${TEST_FOLDER}/file_*, got $RAT_PATH_CUSTOM_NOVOPS_VAR"; return 1; }

	[[ "$(cat $NOVOPS_TEST_APP_FILE_CAT)" == "meow" ]] || { echo "Expected content of $NOVOPS_TEST_APP_FILE_CAT to be 'meow', found '$(cat $NOVOPS_TEST_APP_FILE_CAT)'"; return 1; }
}


# test_chmod() {
#   local TEST_FOLDER
#   TEST_FOLDER=$(mktemp -d)

#   test_start "chmod"

#   (
# 	cd "$TEST_FOLDER" || exit
# 	novops -c "$CONFIG" -l "$LOG_LEVEL" load test-chmod -o env

# 	out=$(stat -c "%a" test-chmod-600.txt)
# 	if [ "$out" != "600" ]; then
# 		echo "invalid chmod value"
# 		return 1
# 	fi

# 	out=$(stat -c "%a" test-chmod-777.txt)
# 	if [ "$out" != "777" ]; then
# 		echo "invalid chmod value"
# 		return 1
# 	fi
#   )
# }

# test_bw_files() {
# 	local TEST_FOLDER=$(mktemp -d)

# 	test_start "bw files"

# 	cd "$TEST_FOLDER"
# 	novops -l "$LOG_LEVEL" -c "$CONFIG" load test-bw-file -o env

# 	if [ ! -f "bw-file" ]; then
# 		echo "Did not create the file from bitwarden"
# 		return 1
# 	fi
# 	cd - > /dev/null
# }

# # Test exit when trying to use a .novops.yml file with BitWarden entries but BitWarden is not unlocked
# test_nobw() {
# 	test_start "BitWarden locked test"

# 	BW_SESSION="" novops load test-bw-file > /dev/null 2>&1
# 	NOVOPS_RETURN=$?
# 	if [ $NOVOPS_RETURN -ne 4 ]; then
# 		echo "Novops usage with unlocked BitWarden session should return exit code 4, got $NOVOPS_RETURN"
# 		return 1
# 	fi
# }

if test_basic; then
	echo "All tests OK"
else
	echo "Some error occured during test, check logs above."
	exit 1
fi
