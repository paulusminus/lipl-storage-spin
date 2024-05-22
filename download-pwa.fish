#!/usr/bin/fish

wget https://github.com/paulusminus/lipl-control/releases/latest/download/lipl-pwa.tar.gz

cd assets
tar -xzf ../lipl-pwa.tar.gz --no-same-owner --no-same-permissions
