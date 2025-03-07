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

curl -X POST $ADDR/tables/1/fields \
    -H "Content-Type: application/json" \
    -d "$json"

     
curl -X POST $ADDR/tables/1/entries \
    -H "Content-Type: application/json" \
    -d '{"1": "Test text", "2": 123}'


curl -X PUT $ADDR/tables/1/entries/1 \
    -H "Content-Type: application/json" \
    -d '{"1": "New text", "2": 666}'

curl -X DELETE $ADDR/tables/1/entries/1

curl -X DELETE $ADDR/tables/1/fields/1

curl -X DELETE $ADDR/tables/1

curl -X GET $ADDR/tables/1/data


set json '{
    "table_id": 1,
    "title": "Test chart",
    "chart_kind": "Bar",
    "axes": [
        {
            "field_id": 1,
            "axis_kind": "X",
            "aggregate": null
        },
        {
            "field_id": 2,
            "axis_kind": "Y",
            "aggregate": "Sum"
        }
    ]
}'

curl -X POST $ADDR/dashboards/1/charts \
    -H "Content-Type: application/json" \
    -d "$json"


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
