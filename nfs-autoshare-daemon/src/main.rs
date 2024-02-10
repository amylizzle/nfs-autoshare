use std::net::{UdpSocket, SocketAddr, TcpListener};
use std::sync::RwLock;
use std::thread;
use std::time::{SystemTime, Duration};
use std::io::{Read, Write};
use std::collections::HashMap;
use ipnetwork::{Ipv4Network, Ipv6Network};
use once_cell::sync::Lazy;

const CONFIG_BROADCAST_PORT: u16 = 5005;
const CONFIG_BROADCAST_INTERFACE: &str = "0.0.0.0";
const CONFIG_DEBUG_PRINTS: bool = true;

static AVAILABLE_IMPORTS: Lazy<RwLock<HashMap<Export, SystemTime>>> = Lazy::new(RwLock::default);

#[derive(PartialEq, Eq, Hash)]
struct Export {
    address: String,
    mount_point: String,
}


fn broadcast_server(socket: &UdpSocket) {
    //set up the socket for broadcasting
    socket.set_broadcast(true).expect("set_broadcast call failed on broadcast_server thread");
    //read the export table
    let export_table = match std::fs::read_to_string("/var/lib/nfs/etab"){
        Ok(table) => table,
        Err(_) => {
            if CONFIG_DEBUG_PRINTS {
                println!("Failed to read export table. No exports to broadcast.");
            }
            return
        },
    };

    //send the export table to the broadcast address
    for line in export_table.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        let mount_name = parts[0];
        let mount_address = parts[1].split('(').next().unwrap();
        if CONFIG_DEBUG_PRINTS {
            println!("exporting {} to {}", mount_name, mount_address);
        }
        // Calculate the broadcast address
        let broadcast_addr;
        match mount_address.parse::<Ipv4Network>() {
            Ok(ipv4_range) => {
                // Calculate broadcast address for IPv4
                let broadcast_address = ipv4_range.broadcast();
                //create addr from broadcast address and port
                broadcast_addr = SocketAddr::new(std::net::IpAddr::V4(broadcast_address), CONFIG_BROADCAST_PORT);
                if CONFIG_DEBUG_PRINTS {
                    println!("Broadcast address (IPv4): {}", broadcast_address);
                }
            }
            Err(_) => {
                // Parsing as IPv4 failed, attempt parsing as IPv6
                match mount_address.parse::<Ipv6Network>() {
                    Ok(ipv6_range) => {
                        // Calculate broadcast address for IPv6
                        let broadcast_address = ipv6_range.broadcast();
                        //create addr from broadcast address and port
                        broadcast_addr = SocketAddr::new(std::net::IpAddr::V6(broadcast_address), CONFIG_BROADCAST_PORT);
                        if CONFIG_DEBUG_PRINTS {
                            println!("Broadcast address (IPv6): {}", broadcast_address);
                        }
                    }
                    Err(e) => {
                        panic!("Failed to parse as IPv6 or IPv4: {}", e);
                    }
                }
            }
        };

        // Calculate the broadcast address
        
        if CONFIG_DEBUG_PRINTS {
            println!("sending on {}", broadcast_addr);
        }
        socket.send_to(mount_name.as_bytes(), broadcast_addr).unwrap();
    }
}

fn broadcast_client(socket: &UdpSocket, export_table: &RwLock<HashMap<Export, SystemTime>>){
    let mut data = [0; 1024];
    let (size, addr) = socket.recv_from(&mut data).expect("Didn't receive data");
    let maybe_export = String::from_utf8_lossy(&data[..size]);
    if CONFIG_DEBUG_PRINTS{
        print!("Received: {} from {}", maybe_export, addr);
    }
    export_table.write().unwrap().insert(Export{address: addr.to_string(), mount_point: maybe_export.to_string()}, SystemTime::now());
}

fn config_server(export_table: &RwLock<HashMap<Export, SystemTime>>){
    let listener = match TcpListener::bind("127.0.0.1:59576") {
        Ok(listener) => listener,
        Err(_) => panic!("Failed to bind config listener"),
    };

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut data = [0; 1024];
        stream.read(&mut data).unwrap();
        if CONFIG_DEBUG_PRINTS {
            println!("config received: {:?}", data);
        }

        let mut result = Vec::new();
        let mut exports = export_table.write().unwrap();
        exports.retain(|_, last_seen| {
            SystemTime::now().duration_since(*last_seen).unwrap().as_secs() < 30
        });
        exports.keys().for_each(|export| {
            result.push(format!("{}:{}", export.address, export.mount_point));
        });
        let response = result.join("\n");
        stream.write(response.as_bytes()).unwrap();
    }
}

fn main() {
    let addr = &format!("{}:{}", CONFIG_BROADCAST_INTERFACE, CONFIG_BROADCAST_PORT);
    let socket= UdpSocket::bind(addr).expect(&format!("Failed to bind broadcast socket to {}",addr));
    println!("Listening on {}", addr);

    let server_send_socket = socket.try_clone().unwrap();
    let server_send_thread = thread::spawn(move || {
        loop {
            broadcast_server(&server_send_socket);
            thread::sleep(Duration::from_secs(10));
        }
    }); 
    let server_recieve_socket = socket.try_clone().unwrap();
    let server_receive_thread = thread::spawn(move || {
        loop {
            broadcast_client(&server_recieve_socket, &AVAILABLE_IMPORTS);
        }
    });

    let config_thread = thread::spawn(move || {
        loop{
            config_server(&AVAILABLE_IMPORTS)
        }
    });

    server_send_thread.join().unwrap();
    server_receive_thread.join().unwrap();
    config_thread.join().unwrap();
}

  

