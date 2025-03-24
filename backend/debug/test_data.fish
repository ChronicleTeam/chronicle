set ADDR 'http://localhost:3000/api'

set json '{
    "name": "Enum field",
    "field_kind": {
        "type": "Enumeration",
        "is_required": true,
        "values": {
            "1": "val1",
            "2": "val2",
            "3": "val3"
        },
        "default_value": 1
    }
}'

set json '{
  "1": 1,
  "2": 0
}'

curl -X PATCH $ADDR/tables/1/fields/order \
    -H "Content-Type: application/json" \
    -d "$json"


curl -X POST $ADDR/tables/excel \
    -F "file=@test.xlsx"

curl -X GET  $ADDR/tables/1/excel \
    -o export.xlsx \
    -F "dummy="


curl -X GET -o export.xlsx $ADDR/tables/1/excel \
    -F "file=@test.xlsx" 

curl -X POST $ADDR/tables/csv \
    -F "file=@test.csv"

curl -X GET $ADDR/tables/1/csv \
    -o export.csv


