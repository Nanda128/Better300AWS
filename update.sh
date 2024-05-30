#!/bin/bash

git stash

git pull origin main

# update teh paths for cargo
source $HOME/.cargo/env

# update teh converter
cargo build --release

# restart
sudo systemctl restart better-300.service