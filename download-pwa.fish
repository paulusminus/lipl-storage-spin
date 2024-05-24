#!/usr/bin/fish

wget https://github.com/paulusminus/lipl-control/releases/latest/download/lipl-pwa.tar.gz

mkdir -p pwa
cd pwa
rm -rf *
tar -xzf ../lipl-pwa.tar.gz --no-same-owner --no-same-permissions
rm ../lipl-pwa.tar.gz
