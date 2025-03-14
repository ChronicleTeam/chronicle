#!/bin/bash

set ADDR 'http://localhost:3000/api'

curl -X POST $ADDR/tables \
    -H "Content-Type: application/json" \
    -d '{"name": "Test Table", "description": "This is a test table"}'


curl -X POST $ADDR/tables/1/fields \
    -H "Content-Type: application/json" \
    -d '{"name": "Field 1", "field_kind": {"type": "Text", "is_required": true}}'

curl -X POST $ADDR/tables/1/fields \
    -H "Content-Type: application/json" \
    -d '{"name": "Field 2", "field_kind": {"type": "Integer", "is_required": true}}'

curl -X PATCH $ADDR/tables/1/fields/order \
    -H "Content-Type: application/json" \
    -d '{"1": 1, "2": 2}'


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
  "name": "Field 13233333",
  "field_kind": {
    "type": "Float",
    "is_required": true,
    "range_start": 10,
    "range_end": 100,
    "scientific_notation": false,
    "number_scale": 2,
    "number_precision": 6
  }
}'

curl -X POST $ADDR/tables/1/fields \
    -H "Content-Type: application/json" \
    -d "$json"

     
curl -X POST $ADDR/tables/1/entries \
    -H "Content-Type: application/json" \
    -d '{"1": "Test text", "2": 123}'


curl -X PUT $ADDR/tables/1/entries/1 \
    -H "Content-Type: application/json" \
    -d '{"1": "New text", "2": 666}'

curl -X GET $ADDR/tables/1/data



curl -X POST $ADDR/tables/1/entries \
    -H "Content-Type: application/json" \
    -d '{"1": "c1", "2": 1}'

curl -X POST $ADDR/tables/1/entries \
    -H "Content-Type: application/json" \
    -d '{"1": "c1", "2": 2}'

curl -X POST $ADDR/tables/1/entries \
    -H "Content-Type: application/json" \
    -d '{"1": "c1", "2": 3}'

curl -X POST $ADDR/tables/1/entries \
    -H "Content-Type: application/json" \
    -d '{"1": "c1", "2": 4}'

curl -X POST $ADDR/tables/1/entries \
    -H "Content-Type: application/json" \
    -d '{"1": "c1", "2": 5}'


curl -X POST $ADDR/tables/1/entries \
    -H "Content-Type: application/json" \
    -d '{"1": "c2", "2": 10}'

curl -X POST $ADDR/tables/1/entries \
    -H "Content-Type: application/json" \
    -d '{"1": "c2", "2": 15}'

curl -X POST $ADDR/tables/1/entries \
    -H "Content-Type: application/json" \
    -d '{"1": "c2", "2": 20}'


curl -X POST $ADDR/tables/1/entries \
    -H "Content-Type: application/json" \
    -d '{"1": "c3", "2": 123}'

curl -X POST $ADDR/tables/1/entries \
    -H "Content-Type: application/json" \
    -d '{"1": "c3", "2": 321}'


curl -X POST $ADDR/tables/export \
    -F "file=@test.xlsx"