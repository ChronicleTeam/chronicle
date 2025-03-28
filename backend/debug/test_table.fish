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


curl -b cookies.txt -X POST $ADDR/tables/$TABLE_ID/entries \
    -H "Content-Type: application/json" \
    -d '{"cells": {"1": "c1", "2": 1}}'

curl -b cookies.txt -X POST $ADDR/tables/$TABLE_ID/entries \
    -H "Content-Type: application/json" \
    -d '{"cells": {"1": "c1", "2": 2}}'

curl -b cookies.txt -X POST $ADDR/tables/$TABLE_ID/entries \
    -H "Content-Type: application/json" \
    -d '{"cells": {"1": "c1", "2": 3}}'

curl -b cookies.txt -X POST $ADDR/tables/$TABLE_ID/entries \
    -H "Content-Type: application/json" \
    -d '{"cells": {"1": "c1", "2": 4}}'

curl -b cookies.txt -X POST $ADDR/tables/$TABLE_ID/entries \
    -H "Content-Type: application/json" \
    -d '{"cells": {"1": "c1", "2": 5}}'

curl -b cookies.txt -X POST $ADDR/tables/$TABLE_ID/entries \
    -H "Content-Type: application/json" \
    -d '{"cells": {"1": "c2", "2": 10}}'

curl -b cookies.txt -X POST $ADDR/tables/$TABLE_ID/entries \
    -H "Content-Type: application/json" \
    -d '{"cells": {"1": "c2", "2": 15}}'

curl -b cookies.txt -X POST $ADDR/tables/$TABLE_ID/entries \
    -H "Content-Type: application/json" \
    -d '{"cells": {"1": "c2", "2": 20}}'


curl -b cookies.txt -X POST $ADDR/tables/$TABLE_ID/entries \
    -H "Content-Type: application/json" \
    -d '{"cells": {"1": "c3", "2": 123}}'

curl -b cookies.txt -X POST $ADDR/tables/$TABLE_ID/entries \
    -H "Content-Type: application/json" \
    -d '{"cells": {"1": "c3", "2": 321}}'

curl -b cookies.txt -X POST $ADDR/tables \
    -H "Content-Type: application/json" \
    -d '{"parent_id": 5, "name": "Child Table", "description": ""}'


set TABLE_ID 8

curl -b cookies.txt -X POST $ADDR/tables/$TABLE_ID/fields \
    -H "Content-Type: application/json" \
    -d '{"name": "Field", "field_kind": {"type": "Text", "is_required": true}}'

curl -b cookies.txt -X POST $ADDR/tables/$TABLE_ID/entries \
    -H "Content-Type: application/json" \
    -d '{"parent_id": 1, "cells": {"3": "test"}}'