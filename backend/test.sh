#!/bin/bash

curl -v -X POST http://localhost:3000/api/tables \
     -H "Content-Type: application/json" \
     -d '{"name": "Test Table", "description": "This is a test table"}'


curl -v -X POST http://localhost:3000/api/tables/1/fields \
     -H "Content-Type: application/json" \
     -d '{"name": "Field 1", "options": {"type": "Text", "is_required": true}}'

curl -v -X POST http://localhost:3000/api/tables/1/fields \
     -H "Content-Type: application/json" \
     -d '{"name": "Field 2", "options": {"type": "Integer", "is_required": true}}'

curl -v -X PUT http://localhost:3000/api/tables/1/fields/1 \
     -H "Content-Type: application/json" \
     -d '{"name": "Field 1 Changed", "options": {"type": "Text", "is_required": false}}'
     
curl -v -X POST http://localhost:3000/api/tables/1/entries \
     -H "Content-Type: application/json" \
     -d '{"1": "Test text", "2": 123}'


curl -v -X PUT http://localhost:3000/api/tables/1/entries/1 \
     -H "Content-Type: application/json" \
     -d '{"1": "Other text", "2": 321}'

curl -v -X DELETE http://localhost:3000/api/tables/1/entries/1

curl -v -X DELETE http://localhost:3000/api/tables/1/fields/1

curl -v -X DELETE http://localhost:3000/api/tables/1