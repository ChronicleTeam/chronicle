#!/bin/bash

set ADDR 'http://localhost:8000/api'

curl -X POST $ADDR/tables \
    -H "Content-Type: application/json" \
    -d '{"name": "Test Table", "description": "This is a test table"}' \
    -b cookies.txt


curl -X POST $ADDR/tables/1/fields \
    -H "Content-Type: application/json" \
    -d '{"name": "Field", "field_kind": {"type": "Text", "is_required": true}}'

curl -X POST $ADDR/tables/1/fields \
    -H "Content-Type: application/json" \
    -d '{"name": "Field", "field_kind": {"type": "Integer", "is_required": true}}'


curl -X POST $ADDR/tables/1/entries \
    -H "Content-Type: application/json" \
    -d '{"cells": {"1": "c1", "2": 1}}'

curl -X POST $ADDR/tables/1/entries \
    -H "Content-Type: application/json" \
    -d '{"cells": {"1": "c1", "2": 2}}'

curl -X POST $ADDR/tables/1/entries \
    -H "Content-Type: application/json" \
    -d '{"cells": {"1": "c1", "2": 3}}'

curl -X POST $ADDR/tables/1/entries \
    -H "Content-Type: application/json" \
    -d '{"cells": {"1": "c1", "2": 4}}'

curl -X POST $ADDR/tables/1/entries \
    -H "Content-Type: application/json" \
    -d '{"cells": {"1": "c1", "2": 5}}'

curl -X POST $ADDR/tables/1/entries \
    -H "Content-Type: application/json" \
    -d '{"cells": {"1": "c2", "2": 10}}'

curl -X POST $ADDR/tables/1/entries \
    -H "Content-Type: application/json" \
    -d '{"cells": {"1": "c2", "2": 15}}'

curl -X POST $ADDR/tables/1/entries \
    -H "Content-Type: application/json" \
    -d '{"cells": {"1": "c2", "2": 20}}'


curl -X POST $ADDR/tables/1/entries \
    -H "Content-Type: application/json" \
    -d '{"cells": {"1": "c3", "2": 123}}'

curl -X POST $ADDR/tables/1/entries \
    -H "Content-Type: application/json" \
    -d '{"cells": {"1": "c3", "2": 321}}'

curl -X POST $ADDR/tables \
    -H "Content-Type: application/json" \
    -d '{"parent_id": 1, "name": "Child Table", "description": ""}'


curl -X POST $ADDR/tables/2/fields \
    -H "Content-Type: application/json" \
    -d '{"name": "Field", "field_kind": {"type": "Text", "is_required": true}}'

curl -X POST $ADDR/tables/2/entries \
    -H "Content-Type: application/json" \
    -d '{"parent_id": 1, "cells": {"3": "test"}}'