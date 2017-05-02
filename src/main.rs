use std::env;
use std::io::{Read, Write, BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::thread;
use std::sync::Mutex;
use std::ffi::OsString;
use std::fs::{self, DirEntry,File};

const SERVER_NAME: &'static str = "agf453-aaronleon-web-server/0.1";

fn main() {
	let listener = TcpListener::bind("127.0.0.1:8082").unwrap();

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

//  reads the client's requests and stops reading at an empty line (at the end of the request header)
fn handle_client(stream: TcpStream) {
    let mut reader = BufReader::new(stream);
	let mut request = String::new();
	let mut is_first_line = true;
    for line in reader.by_ref().lines() {
		// Checking if this is the first line of the client request - i.e the line containing the request parameters
		let val = line.unwrap();
		if is_first_line == true{
			request = val.clone();
			is_first_line = false;
		}
        if val == "" {
            //  At the end of the header we reset the is_first_line var
			break;
        }
    }
    send_response(reader.into_inner(), request);
}

fn send_response(mut stream: TcpStream, req: String) {
	let request = req.clone();
	//Based on the type of the request, we send the appropriate response - 200, 400, 403 or 404
	let status = status_code(request);
	// let response_obj = form_response(status, req.clone());
    let response = "HTTP/1.1 200 OK\n\n<html><body>Hello, World!</body></html>";
    stream.write_all(response.as_bytes()).expect("Returning HTTP response failed.");
}


fn status_code(req: String) -> u32 {
	let tokens:Vec<&str> = req.split(' ').collect();
	//  Improperly formatted GET request: (return status code 400)
	//  a) The HTTP request is not a GET request
	//  b) The request has more/less than 3 spaces 
	//  c) The last space separated word in the request is either HTTP or HTTP/1._ (anything more than 0.9)
	
	// Check if the request has more/less than 3 spaces 
	if tokens.len() != 3 {
		return 400;
	}

	// Check if request method is GET
	if tokens[0] != "GET" {
		return 400;
	}

	// Check if request is using HTTP protocol and version number is greater than 0.9
	let version = tokens[2];
	if version.len() > 4 {
		let (protocol, version_number) = version.split_at(4);
		if protocol != "HTTP" || version_number.parse::<f64>().unwrap() < 0.9 {
			return 400;
		}
	}
	else if version != "HTTP" { return 400; }
	
	//  Checking now if the file requested is either existing and/OR accessible :---> 
	let path_name = tokens[1];
	let path = env::current_dir().unwrap()
		.as_path()
		.join(Path::new(path_name));
	
	// In case the requested file isn't acccessible then we return a 403 Forbidden error
	if !path.is_file() && !path.is_dir(){
		return 403;
	}
	// In case the requested file doesn't exist we returrn the 404 Not Found error
	if !path.exists() || !path.has_root() {
		return 404;
	}
	
	// If there are no errors in the request and the file requested is both existing as well accessible then return 200 OK 
	200
}

//  Depending on the status code we form the appropriate response
fn form_response(code: u32, req: String) -> String {
	let return_string = "HTTP/1.0".to_string();
	//  In case we get an error or a code which was not 200 we return as needed
	if code != 200 {
		if code == 400{
			return return_string + &(code.to_string()) + "Bad Request";
		}
		else if code == 403{
			return return_string + &(code.to_string()) + "Forbidden";
		}
		else if code == 404{
			return return_string + &(code.to_string()) + "Not Found";
		}
	}
	//  In case the status code is 200 then we extract the correct data from the request and display the response
	let tokens:Vec<&str> = req.split(' ').collect();
	let mut path_ext:String = String::new();
	let path_name = tokens[1];
	let path = env::current_dir().unwrap()
		.as_path()
		.join(Path::new(path_name));
	
	let mut file_name:PathBuf = (*(Path::new("defualt"))).to_path_buf();
	//  Find the appropriate file name and then get its details
	if path.is_file(){
		file_name = path.to_path_buf();
	}else if path.is_dir(){
		// let mut base_path = path.clone();
		let options:Vec<&str> = vec!["/index.html", "/index.shtml","/index.txt"];
		let mut path_buf = path.to_path_buf();
		let mut temp_path:PathBuf = (*(Path::new("defualt"))).to_path_buf();
		
		for index in 0..options.len(){
			// temp_path = path_buf.clone();
			path_buf.push(options[index]);
			temp_path = path_buf.clone();
			if temp_path.exists(){
				file_name = temp_path.clone();
				break;
			}
			temp_path = (*path_buf.parent().unwrap()).to_path_buf();
		}
	};
	//  Find the various parameters of the file, required to build a proper return object:--- 
	let file_path = file_name;
	//  Find the extension of the file inputted
	if file_path.extension().unwrap() == "html"{
		path_ext = "text/html".to_string();
	}else{
		path_ext = "text/plain".to_string();
	}
	
	//  Find the size of the file 
	// let bytes = calculate_bytes(file_path.clone());
	
	// Find the data in the file
	// let file_content = content(file_path.clone());
	
	return return_string + &(code.to_string())
							+ "OK \n" + SERVER_NAME + "\n Content-type:" + &path_ext ;
							// + "\nContent-length:" + &(bytes.to_string()) 
							// +"\n" + &file_content;
}

//  Find the number of bytes in the file
fn calculate_bytes(path: PathBuf) -> usize{
	let mut buffer = Vec::new();
	let f = File::open(path);
	// read the whole file
	f.unwrap().read_to_end(&mut buffer);
	// Output the length
	return buffer.len();
}

//  Find the contents of the file, to display to the users
fn content(path:PathBuf) -> String{
	let path_buf = path.read_link().expect("read_link call failed");
	let os = path_buf.into_os_string();
	return os.into_string().unwrap();
}