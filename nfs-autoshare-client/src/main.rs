use std::net::TcpStream;
use std::io::{Read, Write};

fn get_exports() -> Vec<String> {
    let mut sock = TcpStream::connect("127.0.0.1:59576").expect("Failed to connect to server");
    sock.write(b"list\n").expect("Failed to send data to server");
    let mut data = String::new();
    sock.read_to_string(&mut data).expect("Failed to receive data from server");
    let exports: Vec<String> = data.trim().split("\n").map(|s| s.to_string()).collect();
    exports
}

fn try_mount(host: &str, share: &str, mount_point: &str) -> String {
    let mut sock = TcpStream::connect("127.0.0.1:59576").expect("Failed to connect to server");
    let command = format!("mount {} {} {}", host, share, mount_point);
    sock.write(command.as_bytes()).expect("Failed to send data to server");
    let mut data = String::new();
    sock.read_to_string(&mut data).expect("Failed to receive data from server");
    return data;
}

fn main() {
    let export_list = get_exports();
    println!("{:?}", export_list);
    if export_list[0] == "invalid command" {
        println!("Failed to get exports");
        std::process::exit(1);
    }
    if export_list.is_empty() || export_list[0].is_empty() {
        println!("No shares available");
        std::process::exit(1);
    }
    println!("Available shares:");
    for (i, export) in export_list.iter().enumerate() {
        let parts: Vec<&str> = export.split(":").collect();
        let host = parts[0];
        let share = parts[1];
        println!("[{}]: {} on {}", i, share, host);
    }

    println!("Enter the number of the share you want to mount:");
    let choice: usize = {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read input");
        input.trim().parse().expect("Invalid input")
    };

    if choice >= export_list.len() {
        println!("Invalid choice");
        std::process::exit(1);
    }

    let parts: Vec<&str> = export_list[choice].split(":").collect();
    let host = parts[0];
    let share = parts[1];
    let default_mount_point = format!("/media/{}{}", host, share);

    println!("Where would you like to mount {} on {}? ({}): ", share, host, default_mount_point);
    let mut mount_point = String::new();
    std::io::stdin().read_line(&mut mount_point).expect("Failed to read input");
    let mount_point = mount_point.trim().to_string();
    let mount_point = if mount_point.is_empty() {
        default_mount_point
    } else {
        mount_point
    };

    println!("Mounting {} on {} to {}", share, host, mount_point);
    let result = try_mount(host, share, &mount_point);
    println!("{}", result);
}
