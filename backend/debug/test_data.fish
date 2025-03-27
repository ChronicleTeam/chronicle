set ADDR 'http://localhost:8000/api'

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


# Import excel
curl -X POST $ADDR/tables/excel \
    -F "file=@data/test.xlsx"

# Export excel without existing spreadsheet
curl -X GET  $ADDR/tables/1/excel \
    -o data/export.xlsx \
    -F "dummy="

# Export excel with existing spreadsheet
curl -X GET $ADDR/tables/1/excel \
    -o data/export.xlsx \
    -F "file=@data/test.xlsx" 

# Import CSV
curl -X POST $ADDR/tables/csv \
    -F "file=@data/import.csv"

# Export CSV
curl -X GET $ADDR/tables/1/csv \
    -o data/export.csv


