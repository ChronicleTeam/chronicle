set ADDR 'http://localhost:8000/api'

curl -b cookies.txt -X GET $ADDR/tables/1/data -v

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

curl -b cookies.txt -X PATCH $ADDR/tables/1/fields/order \
    -H "Content-Type: application/json" \
    -d "$json"


# Import excel
curl -b cookies.txt -X POST $ADDR/tables/excel \
    -F "file=@data/import.xlsx"

# Export excel without existing spreadsheet
curl -b cookies.txt -X GET  $ADDR/tables/1/excel \
    -o data/export.xlsx \
    -F "dummy="

# Export excel with existing spreadsheet
curl -b cookies.txt -X GET $ADDR/tables/1/excel \
    -o data/export.xlsx \
    -F "file=@data/test.xlsx" 

# Import CSV
curl -b cookies.txt -X POST $ADDR/tables/csv \
    -F "file=@data/import.csv"

# Export CSV
curl -b cookies.txt -X GET $ADDR/tables/1/csv \
    -o data/export.csv


