#!/bin/sh

TEST=false
USER=""
PASS=""
while getopts ":hp:tu:" Option
do
	case $Option in
		t ) TEST=true;;
	esac
done

while [ $# -gt 0 ]; do
	case "$1" in
		--username*|-u*)
			if [[ "$1" != *=* ]]; then shift; fi # Value is next arg if no `=`
			USER="${1#*=}";;
		--password*|-p*)
			if [[ "$1" != *=* ]]; then shift; fi
			PASS="$(echo "${1#*=}" | jq -Rr @uri)"
			echo $PASS;;
		--help|-h)
			echo "Meaningful help message"
			exit 0;;
	esac
	shift
done

# Set up script environment
FOLDER="/mnt/onboard/.adds/ao3-reader/ao3offline"
SQLITE="${FOLDER}/sqlite3"
JQ="${FOLDER}/jq"
CURL="${FOLDER}/curl"

# Note: When testing, assume sqlite, jq, and curl are installed locally
if [[ $TEST = true ]]
then
	FOLDER="."
	SQLITE="sqlite3"
	JQ="jq"
	CURL="curl"
fi

SITE="archiveofourown.org"
COOKIE_FOLDER="${FOLDER}/cookies"
COOKIE_FILE="${COOKIE_FOLDER}/${SITE}.txt"

mkdir -p "${COOKIE_FOLDER}"

# Always start a new session when logging in
TOKEN=$(curl -s -b "${COOKIE_FILE}" "https://${SITE}/token_dispenser.json" | jq -r '.token') && echo "${TOKEN}"
LOGIN_RESULT=$(curl "https://${SITE}/users/login" \
	-X POST -s \
	-b "${COOKIE_FILE}" \
	--data-raw "authenticity_token=${TOKEN}&user%5Blogin%5D=${USER}&user%5Bpassword%5D=${PASS}&user%5Bremember_me%5D=1&commit=Log+In")

echo "${LOGIN_RESULT}"

if echo "${LOGIN_RESULT}" | grep -q "${USER}"
then
	echo "Logged in to ${SITE} as ${USER}"
else
	echo "Unable to log in to ${SITE}"
fi
