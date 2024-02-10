import socket
import os
#connect to local server and get the list of available exports
def get_exports():
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.connect(("127.0.0.1", 59576))
    sock.send(b'list\n')
    data = sock.recv(1024)
    sock.close()
    return data.decode().split("\n")

if __name__ == "__main__":
    export_list = get_exports()
    print("Available shares:")
    i=0
    for export in export_list:
        host, share = export.split(":")
        print(f'[{i}]: {share} on {host}')
        i+=1
    
    choice = int(input("Enter the number of the share you want to mount: "))
    if(choice < 0 or choice >= len(export_list)):
        print("Invalid choice")
        exit(1)
    host, share = export_list[choice].split(":")
    default_mount_point = f'/media/{host}{share}'
    #Where would you like to mount /mnt/Storage on Alice-PC? (/media/Alice-Pc/mnt/Storage):
    mount_point = input(f'Where would you like to mount {share} on {host}? ({default_mount_point}): ')
    if(mount_point == ""):
        mount_point = default_mount_point
    print(f'Mounting {share} on {host} to {mount_point}')
    #mount the share
    os.system(f'sudo mkdir -p {mount_point} && sudo mount -t nfs {host}:{share} {mount_point}')
