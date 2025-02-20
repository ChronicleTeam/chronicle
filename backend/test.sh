#!/bin/bash

curl -v -X POST http://localhost:3000/api/tables \
     -H "Content-Type: application/json" \
     -d '{"name": "Test Table", "description": "This is a test table"}'


curl -v -X POST http://localhost:3000/api/tables/1/fields \
     -H "Content-Type: application/json" \
     -d '{"name": "Field 1", "options": {"type": "Text", "is_required": true}}'
     
curl -v -X POST http://localhost:3000/api/tables/1/entries \
     -H "Content-Type: application/json" \
     -d '{"2": "Test text", "1": 123}'
