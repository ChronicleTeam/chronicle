#!/bin/bash

curl -v -X POST http://localhost:3000/api/tables/1/fields \
     -H "Content-Type: application/json" \
     -d '{"name": "Test Field", "options": {"type": "text", "is_required": false} }'

