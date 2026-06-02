#!/bin/sh

time curl -X POST http://localhost:3000/lipl/api/v1/db -u paul -H "Content-Type: application/json" -d @data/db.json
