use std::net::TcpStream;
use std::io::{self, Write, Read};

fn send_command(stream: &mut TcpStream, command: &str) -> io::Result<String> {
    stream.write_all(command.as_bytes())?;
    stream.flush()?;

    let mut buffer = [0; 512];
    let size = stream.read(&mut buffer)?;

    let response = String::from_utf8_lossy(&buffer[..size]);
    Ok(response.to_string())
}

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8080").expect("Could not connect to server");

    loop {
        println!("Enter Redis command (e.g., SET key value or GET key):");
        let mut command = String::new();
        io::stdin().read_line(&mut command).expect("Failed to read line");

        let command = command.trim();

        if command.to_lowercase() == "exit" {
            println!("Exiting the client.");
            break;
        }

        match send_command(&mut stream, &format!("{}\r\n", command)) {
            Ok(response) => {
                println!("Response: {}", response);
            }
            Err(e) => {
                println!("Failed to send command: {}", e);
            }
        }
    }
}
