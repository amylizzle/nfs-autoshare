# IN DEVELOPMENT - LIKELY TO BE BUGGY
# nfs-autoshare
Zero-config sharing of NFS exports

## what is this?
NFS shares are kind of a pain to set up on the client-side. This project seeks to make that easier. Any NFS shares will be advertised to the appropriate network range, and any machine running nfs-autoshare will keep a record of shares it's seen recently. For easy mounting of nfs shares, just run `nfs-autoshare-client` and select the share you want to mount and where to mount it.

## how do I use it?
Follow the build instructions below (or install from a package when I get around to that). Install it on every machine that has NFS shares exported or wants to use those NFS shares.
Wait a few seconds for the advertisements to populate - shouldn't take more than 20 seconds.
run `nfs-autoshare-client` and follow the simple on-screen instructions.

It should look like this: 
```
you@your-pc:~$ sudo nfs-autoshare-client
Available shares:
[0]: /mnt/Storage on 192.168.200.7
Enter the number of the share you want to mount:
0
Where would you like to mount /mnt/Storage on 192.168.200.7? (/media/192.168.200.7/mnt/Storage): 

Mounting /mnt/Storage on 192.168.200.7 to /media/192.168.200.7/mnt/Storage
```


## build instructions
install cargo for your distribution. For debian you'd do this:
```
sudo apt install cargo
```
then clone the repo
```
git clone https://github.com/amylizzle/nfs-autoshare.git
```
change to the directory
```
cd nfs-autoshare
```
and run the build script
```
./build.sh
```
finally, run the install script to add the binaries to sbin and set up the systemd service
```
./install.sh
```
