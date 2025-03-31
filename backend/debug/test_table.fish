#!/bin/bash

set ADDR 'http://localhost:8000/api'

curl -b cookies.txt -X POST $ADDR/tables \
    -H "Content-Type: application/json" \
    -d '{"name": "Test Table", "description": "This is a test table"}'

set TABLE_ID 5

curl -b cookies.txt -X POST $ADDR/tables/$TABLE_ID/fields \
    -H "Content-Type: application/json" \
    -d '{"name": "Field", "field_kind": {"type": "Text", "is_required": true}}'

curl -b cookies.txt -X POST $ADDR/tables/$TABLE_ID/fields \
    -H "Content-Type: application/json" \
    -d '{"name": "Field", "field_kind": {"type": "Integer", "is_required": true}}'


set json '{
  "parent_id": null,
  "entries": [
    {
      "1": "c1",
      "2": 1
    },
    {
      "1": "c1",
      "2": 2
    },
    {
      "1": "c1",
      "2": 3
    },
    {
      "1": "c1",
      "2": 4
    },
    {
      "1": "c1",
      "2": 5
    },
    {
      "1": "c2",
      "2": 10
    },
    {
      "1": "c2",
      "2": 15
    },
    {
      "1": "c2",
      "2": 20
    },
    {
      "1": "c3",
      "2": 123
    },
    {
      "1": "c3",
      "2": 321
    }
  ]
}'

curl -b cookies.txt -X POST $ADDR/tables/$TABLE_ID/entries \
    -H "Content-Type: application/json" \
    -d "$json"

curl -b cookies.txt -X POST $ADDR/tables \
    -H "Content-Type: application/json" \
    -d '{"parent_id": 1, "name": "Child Table", "description": ""}'


set TABLE_ID 8

curl -b cookies.txt -X POST $ADDR/tables/$TABLE_ID/fields \
    -H "Content-Type: application/json" \
    -d '{"name": "Field", "field_kind": {"type": "Text", "is_required": true}}'

set json '{
  "parent_id": 1,
  "entries": [
    {
      "3": "test"
    }
  ]
}'

curl -b cookies.txt -X POST $ADDR/tables/$TABLE_ID/entries \
    -H "Content-Type: application/json" \
    -d "$json"