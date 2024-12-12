#!/usr/bin/env bash

# First time setup includes:
#
# Update packages
# Install and enable firewall
# Set up hardware (/boot/firmware/config.txt)
# Install dependencies: nginx, rustup, python dependencies
# Set up services: api, video, website
# Set up nginx

# Upgrade packages over https
sudo apt update
sudo apt install apt-transport-https
sudo apt upgrade -y

# Install, configure and enable firewall
sudo apt install ufw
sudo ufw allow 22
sudo ufw allow 80
sudo ufw --force enable

# TODO: Append hardware pwm configuration to `/boot/firmware/config.txt`

# TODO: Install rustup automatically

# Compile API once, so service starts right away next time
cargo build --release -p server

# Install python dependencies
pip install -r requirements.txt

# TODO: Set up services

# TODO: Setup nginx
