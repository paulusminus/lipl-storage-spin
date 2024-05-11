#!/usr/bin/bash

# time curl -X POST https://lipl-storage-spin-gfqbmw44.fermyon.app/lyric -H "Content-Type: application/json" -d @data/roodkapje.json
# time curl -X POST https://lipl-storage-spin-gfqbmw44.fermyon.app/lyric -H "Content-Type: application/json" -d @data/grachten.json
time curl -X POST https://lipl.paulmin.nl/db -u paul -H "Content-Type: application/json" -d @data/db.json