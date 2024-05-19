#!/usr/bin/bash

time curl -X POST https://lipl.paulmin.nl/db -u paul -H "Content-Type: application/json" -d @data/db.json
