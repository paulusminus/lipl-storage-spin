#!/usr/bin/fish

set -x AUTHENTICATION (echo -n "$LIPL_USERNAME:$LIPL_PASSWORD" | base64)

echo $AUTHENTICATION

bombardier -H "Content-Type: application/json" -H "Authorization: Basic $AUTHENTICATION" http://localhost:3000/lipl/api/v1/lyric
