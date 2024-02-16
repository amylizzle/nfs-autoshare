use std::net::{IpAddr, TcpListener};
use std::sync::RwLock;
use std::thread;
use std::time::SystemTime;
use std::io::{Read, Write};
use std::collections::HashMap;
use once_cell::sync::Lazy;
use local_ip_address::list_afinet_netifas;
use mdns_sd::{Receiver, ServiceDaemon, ServiceEvent, ServiceInfo, DaemonEvent};
use gethostname::gethostname;

const CONFIG_DEBUG_PRINTS: bool = true;
const SERVICE_TYPE: &str = "_nfs._tcp.local.";

static AVAILABLE_IMPORTS: Lazy<RwLock<HashMap<Export, SystemTime>>> = Lazy::new(RwLock::default);
static MY_EXPORTS: Lazy<RwLock<HashMap<String, String>>> = Lazy::new(RwLock::default);

#[derive(PartialEq, Eq, Hash)]
struct Export {
    address: String,
    mount_point: String,
}


fn broadcast_server(mdns: &ServiceDaemon) {
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

    let network_interfaces = list_afinet_netifas().unwrap();

    let host_name = gethostname().into_string().unwrap()+".local";
    let mut active_exports = HashMap::<String,bool>::new();
    //send the export table to the broadcast address
    for line in export_table.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        let mount_name = parts[0];
        let mount_address = parts[1].split('(').next().unwrap();
        if CONFIG_DEBUG_PRINTS {
            println!("exporting {} to {}", mount_name, mount_address);
        }
        active_exports.insert(mount_name.to_string(),true);
        if MY_EXPORTS.read().unwrap().contains_key(mount_name) {
            continue; //already registered, don't need it again
        }
        MY_EXPORTS.write().unwrap().insert(mount_name.to_string(), mount_address.to_string());
        let host_ips = network_interfaces.iter().filter(|(_,ip)| !ip.is_loopback()).map(|(_,ip)| ip.clone()).collect::<Vec<IpAddr>>();
        let service = ServiceInfo::new(
            SERVICE_TYPE,
            &format!("{} on {}", mount_name, host_name),
            &host_name, 
            "",//empty, let auto assign
            2049,
            &[("path",mount_name)][..],
        )
        .expect("Failed to create service - Invalid service info")
        .enable_addr_auto();
        if CONFIG_DEBUG_PRINTS {
            println!("registering {:?}", service);
        }
        mdns.register(service).expect("Failed to register service");
    }
    let mut to_remove = Vec::<String>::new();
    for (mount_name,_) in MY_EXPORTS.read().unwrap().iter() {
        if !active_exports.contains_key(mount_name) {
            if CONFIG_DEBUG_PRINTS {
                println!("unregistering {}", mount_name);
            }
            to_remove.push(mount_name.to_string());            
            mdns.unregister(&format!("{} on {}.", mount_name, host_name)).expect("Failed to unregister service");
        }
    }
    for mount_name in to_remove {
        MY_EXPORTS.write().unwrap().remove(&mount_name);
    }
    println!("done!");
}

fn broadcast_client(mdns: &ServiceDaemon){
    let receiver = mdns.browse(SERVICE_TYPE).expect("Failed to browse");
    println!("waiting for events");
    while let Ok(event) = receiver.recv_timeout(std::time::Duration::from_secs(5)) {
        println!("got event {:?}", event);
        if let ServiceEvent::ServiceResolved(info) = event {
            println!("resolved {:?}", info);
            match info.get_property_val("path"){
                Some(val) => {
                    match val {
                        Some (val) => {
                            let mount_point = core::str::from_utf8(val).unwrap();
                            let new_export = Export{address: info.get_hostname().to_string(), mount_point: mount_point.to_string()};
                            if CONFIG_DEBUG_PRINTS && !AVAILABLE_IMPORTS.read().unwrap().contains_key(&new_export){
                                println!("new import {} on {}", mount_point, info.get_hostname());
                            }
                            AVAILABLE_IMPORTS.write().unwrap().insert(new_export, SystemTime::now());
                        }
                        None => {
                            return;
                        }
                    }
                },
                None => {
                    return;
                }
            }     
        }
    }
}

fn config_server(){
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
        let mut exports = AVAILABLE_IMPORTS.write().unwrap();
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
    let mdns = ServiceDaemon::new().expect("Failed to create daemon");
    let client_mdns = mdns.clone();
   
    let recieve_thread = thread::spawn(move || {
        loop {
            broadcast_client(&client_mdns);
        }
    });

    let broadcast_thread = thread::spawn(move || {
        loop {
            broadcast_server(&mdns);
            thread::sleep(std::time::Duration::from_secs(5));
        }
    });


    let config_thread = thread::spawn(move || {
        loop{
            config_server()
        }
    });


    recieve_thread.join().unwrap();
    broadcast_thread.join().unwrap();
    config_thread.join().unwrap();
}

  

