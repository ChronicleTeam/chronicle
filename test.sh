#!/bin/bash

curl -v -X POST http://localhost:3000/api/tables \
     -H "Content-Type: application/json" \
     -d '{"name": "My Table", "description": "This is a test table"}'

