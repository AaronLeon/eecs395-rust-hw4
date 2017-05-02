use std::env;
use std::io::{Read, Write, BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::path::{Path};
use std::thread;
use std::sync::Mutex;

fn main() {
	let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

	// accept connections and process them serially
	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				handle_client(stream);
			}
			Err(error) => {
 				println!("{}", error);
			}
		}
	}
}

fn handle_client(stream: TcpStream) {
    let mut reader = BufReader::new(stream);

    for line in reader.by_ref().lines() {
        if line.unwrap() == "" {
            break;
        }
    }
    send_response(reader.into_inner());
}


fn check_http_request(req: String) -> bool {
	let tokens:Vec<&str> = req.split(' ').collect();

	if tokens.len() != 3 {
		return false;
	}

	// Check if request method is GET
	if tokens[0] != "GET" {
		return false;
	}

	let path_name = tokens[1];
	let path = env::current_dir().unwrap()
		.as_path()
		.join(Path::new(path_name));
	if !path.exists() || !path.has_root() {
		return false;
	}

	// Check if request is using HTTP protocol and version number is greater than 0.9
	let version = tokens[2];
	if version.len() > 4 {
		let (protocol, version_number) = version.split_at(4);
		if protocol != "HTTP" || version.parse::<f64>().unwrap() < 0.9 {
			return false;
		}
	}
	else if version != "HTTP" { return false; }

	true
}

fn send_response(mut stream: TcpStream) {
    let response = "HTTP/1.1 200 OK\n\n<html><body>Hello, World!</body></html>";
    stream.write_all(response.as_bytes()).unwrap();
}
