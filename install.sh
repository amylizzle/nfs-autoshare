#!/bin/bash

sudo cp ./nfs-autoshare-client/target/release/nfs-autoshare-client /usr/sbin/
sudo cp ./nfs-autoshare-daemon/target/release/nfs-autoshare-daemon /usr/sbin/
sudo cp ./nfs-autoshare-daemon/nfs-autoshare.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable nfs-autoshare
sudo systemctl start nfs-autoshare
