#!/usr/bin/fish

mkdir -p pwa
cd pwa
rm -rf *
cp -dpr $HOME/Code/dart/lipl_control/build/web/* .
