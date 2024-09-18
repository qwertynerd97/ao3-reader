#!/bin/sh

NOTES="/mnt/onboard/.adds/ao3offline"
SQLITE="${NOTES}/sqlite3"
JQ="${NOTES}/jq"
CURL="${NOTES}/curl"

KOBO="/mnt/onboard"
DB="${KOBO}/.kobo/KoboReader.sqlite"
BOOKS="${KOBO}/.kobo/kepub"
EXPORT_FOLDER="${NOTES}/offlinedata"

LD_LIBRARY_PATH="${NOTES}/lib:${LD_LIBRARY_PATH}"
export LD_LIBRARY_PATH

mkdir -p "${EXPORT_FOLDER}"

# Load title and ID of most recently read book
# This script assums it is being run from inside a book
# and that the most recently read book is the current book
# TODO - this DEFINTELY does not work correctly
SQL="SELECT
  c.Title
  FROM Content c
  WHERE c.ContentType = 6
    AND c.ReadStatus = 1
  ORDER BY c.LastTimeStartedReading DESC limit 1"

TITLE=$($SQLITE "$DB" "$SQL")
echo "Most recently read book is ${TITLE}"
JSON=$(echo "{}" | $JQ --arg value "${TITLE}" '. + {title: $value}')

SQL="SELECT
    c.ContentId
    FROM Content c
    WHERE c.ContentType = 6
      AND c.ReadStatus = 1
    ORDER BY c.LastTimeStartedReading DESC Limit 1;"

ID=$($SQLITE "$DB" "$SQL")
JSON=$(echo "${JSON}" | $JQ --arg value "${ID}" '. + {koboId: $value}')

# Figure out which flags are run
KUDOS=false
COMMENT=false
READ_LATER=false
MARK_READ=false
WIFI=false
while getopts ":kclrw" Option
do
  case $Option in
    k ) KUDOS=true;;
    c ) COMMENT=true;;
    l ) READ_LATER=true;;
    r ) MARK_READ=true;;
    w ) WIFI=true;;
  esac
done

# Check to see if AO3 server is up
TOKEN=""
if [[ $WIFI = true ]]
then
  echo "Loading AO3 Token..."
  TOKEN=$(curl -s -c cookies.txt "https://archiveofourown.org/token_dispenser.json" | $JQ -r '.token')
  echo "${TOKEN}"
fi

# Determine if there is already offline data avalible for this book
EXPORT="${EXPORT_FOLDER}/${ID}.json"
SITE=""
WORK_ID=""
if [[ -e "${EXPORT}" ]]
then
  echo "File ${EXPORT} already exists, loading data"
  JSON=$($JQ '.' "${EXPORT}" || $JSON)
else
  echo "Creating json data..."
  URL=$(unzip -p "${BOOKS}/${ID}" | grep -o -m 1 'https://archiveofourown.org/works/[0-9]*' | head -1)
  JSON=$(echo "${JSON}" | $JQ --arg value "${URL}" '. + {url: $value}')
  NO_PROTOCOL="${URL#*//}"
  JSON=$(echo "${JSON}" | $JQ '. + {actions: []}')
fi

OFFLINE=true
if [[ -n "${TOKEN}" && -n "${URL}" ]]
then
  echo "Kobo is online, data will be sent directly to AO3"
  OFFLINE=false
fi

if [[ $KUDOS = true ]]
then
  JSON=$(echo "${JSON}" | $JQ '.actions[.actions| length] |= . + {"type": "kudos"}')
fi

if [[ $COMMENT = true ]]
then
  # TODO - comments currently do not work properly

  # UTF-8 char table (decimal):
  #      9: Tab
  #     10: Line feed
  #     32: Space
  #     58: :
  #     62: >
  #     42: *
  #     92: \
  #   8230: …
  #   9999: ✏
  # 128278: 🔖
  # 128196: 📄

  SQL="SELECT TRIM(
    '### ' ||
    CASE
      WHEN b.Type = 'dogear' THEN
        char(128278, 32)
      WHEN b.Type = 'note' THEN
        char(9999, 32)
      WHEN b.Type = 'highlight' THEN
        char(128196, 32)
    END
    || c.Title || ', ' || COALESCE(c.Attribution, 'N/A') || char(10, 10, 42)
    || datetime(b.dateCreated) || char(42, 92, 10) /* force Markdown newline */
    || COALESCE(c1.Title, '') || char(10, 10, 62, 32) ||
    CASE
      WHEN b.Type = 'dogear' THEN
        COALESCE(ContextString, 'No context available') || char(8230, 10) /* only kepubs have context */
      ELSE
        REPLACE(              /* start Markdown quote */
          REPLACE(
            TRIM(             /* trim newlines */
              TRIM(           /* trim tabs */
                TRIM(b.Text), /* trim spaces */
                char(9)
              ),
              char(10)
            ),
            char(9), ''
          ),
          char(10), char(10, 62, 32, 10, 62, 32)) /* continue Markdown quote for multiple paragraphs */
        || char(10, 10)
        || COALESCE(b.Annotation, '') || char(10)
    END, char(10)
    ) || char(10, 10)
    FROM (SELECT * FROM Bookmark WHERE VolumeID = '$ID') b
      INNER JOIN content c ON b.VolumeID = c.ContentID
      LEFT OUTER JOIN content c1 ON (c1.ContentId LIKE b.ContentId || '%')
    ORDER BY b.DateCreated ASC;"

  # $SQLITE "$DB" "$SQL" >> $EXPORT

  # echo "## End Comment" >> $EXPORT
fi

if [[ $READ_LATER = true ]]
then
  JSON=$(echo "${JSON}" | $JQ '.actions[.actions| length] |= . + {"type": "readLater"}')
fi

if [[ $MARK_READ = true ]]
then
  JSON=$(echo "${JSON}" | $JQ '.actions[.actions| length] |= . + {"type": "read"}')
fi

if [[ $OFFLINE = true ]]
then
  echo "Saving ${JSON} to ${EXPORT}"
  echo "${JSON}" > $EXPORT
else
  echo "Sending data to AO3..."
  echo "${JSON}" | $JQ '.actions[].type' | while read -r action 
  do 
    case $action in
      kudos     ) echo "Sending kudos";;
      readLater ) echo "Marking for later";;
      read      ) echo "Marking as read";;
    esac
  done
fi

