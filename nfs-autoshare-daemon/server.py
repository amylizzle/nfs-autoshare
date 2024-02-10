config_broadcast_port = 5005
config_broadcast_interface = None # None for all interfaces, or a specific IP 
config_listen_interface = "0.0.0.0" # None for all interfaces, or a specific IP
config_debug_prints = True

import socket
from time import sleep, time
import threading

if config_broadcast_interface is None:
    config_broadcast_interface = socket.gethostname()
if config_listen_interface is None:
    config_listen_interface = socket.gethostname()

available_exports = {}
available_exports_lock = threading.Lock()

def server_send():
    interfaces = socket.getaddrinfo(host=config_broadcast_interface, port=None, family=socket.AF_INET)
    allips = set([ip[-1][0] for ip in interfaces])

    export_table = open("/var/lib/nfs/etab","r")
    export_table.seek(0)
    exports = export_table.readlines()
    for line in exports:
        parts = line.split('\t')
        mount_name = parts[0]
        mount_address = parts[1].partition("(")[0]
        if config_debug_prints:
            print(f'exporting {mount_name} to {mount_address}')

        for ip in allips:
            if config_debug_prints:
                print(f'sending on {ip}')
            sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)  # UDP
            sock.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)
            sock.bind((ip,0))
            sock.sendto(mount_name.encode(), ("255.255.255.255", config_broadcast_port))
            sock.close()

def server_recieve(listensock: socket.socket):
    while True:
        data, addr = listensock.recvfrom(1024)
        available_exports_lock.acquire()
        try:
            if(config_debug_prints):
                print("recieved:",data, addr)
            if(addr[0] in available_exports):
                available_exports[addr[0]][data.decode()] = time()
            else:
                available_exports[addr[0]] = {data.decode(): time()}
            if(config_debug_prints):
                print(available_exports)
        finally:
            available_exports_lock.release()

def config_socket_thread(configsock: socket.socket):
    configsock.listen(1)
    while True:
        conn, addr = configsock.accept()
        data = conn.recv(1024)
        if(config_debug_prints):
            print("config recieved:",data)
        available_exports_lock.acquire()
        try:
            result = []
            for(addr, exports) in list(available_exports.items()):
                for (mount_point,lastseen) in exports.items():
                    if time() - lastseen > 10:
                        del available_exports[addr][mount_point]
                    else:
                        result.append(f'{addr}:{mount_point}')
            conn.send("\n".join(result).encode())
        finally:
            available_exports_lock.release()
        conn.close()


if __name__ == "__main__":
    listensock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    listensock.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)
    listensock.bind((config_listen_interface, config_broadcast_port))

    configsock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    configsock.bind(("127.0.0.1", 59576))
    configthread = threading.Thread(target=config_socket_thread, args=(configsock,))
    configthread.start()

    listenthread = threading.Thread(target=server_recieve, args=(listensock,))
    listenthread.start()

    while True:
        server_send()
        sleep(2)

