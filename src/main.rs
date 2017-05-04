use std::env;
use std::io::{Read, Write, BufRead, BufReader, ErrorKind};
use std::path::Path;
use std::fs::File;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

extern crate chrono;
use chrono::prelude::*;

#[derive(Debug, PartialEq, Eq)]
struct Request {
    method: String,
    path: String,
    protocol: String,
}

#[derive(Debug, PartialEq, Eq)]
struct Response {
    status: String,
    web_server: String,
    content_type: String,
    content_length: usize,
    data: String,
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    let log_file = File::create("server.log").unwrap();
    let log = Arc::new(Mutex::new(log_file));

    for stream in listener.incoming() {
        let log = log.clone();
        match stream {
            Ok(mut s) => {
                thread::spawn(move | | {
                    handle_client(&mut s, &log);
                });
            }
            Err(error) => {
                println!("{}", error);
            }
        }
    }
}

//  reads the client's requests and stops reading at an empty line (at the end of the request header)
fn handle_client(stream: &mut TcpStream, log: &Arc<Mutex<File>>) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut buffer = String::new();
    reader.read_line(&mut buffer).unwrap();
    for line in reader.by_ref().lines() {
        if line.unwrap() == "" {
           break; 
        }
    }
    if let Ok(req) = parse_request(&buffer) {
        let res = handle_request(&req);
        send_response(stream, &res);
        log_request(log, &req, &res)
    }
}

fn parse_request(req_string: &str) -> Result<Request, &'static str> {
    let tokens:Vec<&str> = req_string.split_whitespace()
        .collect();
    if tokens.len() == 3 {
        let req = Request {
            method: tokens[0].to_string(),
            path: tokens[1].to_string(),
            protocol: tokens[2].to_string(),
        };
        return Ok(req);
    }
    return Err("Error! Invalid request length");
}

fn handle_request(req: &Request) -> Response {
    let mut path = env::current_dir().unwrap();
    let req_path = Path::new(&req.path);
    if !is_valid_method(&req.method) || !is_valid_protocol(&req.protocol) || !req_path.has_root() {
        return create_error_response("400");
    }

    let relative_path = req_path.strip_prefix("/").unwrap();
    path = path.join(relative_path);

    if path.is_dir() {
        let default_files = vec!["index.txt", "index.html", "index.shtml"];
        for file in default_files {
            let default_file_path = path.join(file);
            if default_file_path.exists() {
                path = default_file_path;
                break;
            }
        }
    }
    if !path.is_file() {
        return create_error_response("404");
    }

    let file = read_file(&path);
    match file {
        Ok(content) => {
            let extension = path.extension().unwrap().to_str().unwrap();
            let content_type = if extension == "html" { "html" } else { "plain" };
            return create_success_response(&content_type.to_string(), content.capacity(), &content);
        },
        Err(err) => {
            if err == ErrorKind::NotFound {
                return create_error_response("404");
            }
            else if err == ErrorKind::PermissionDenied {
                return create_error_response("403");
            }
            return create_error_response("400");
        },
    }
}

fn is_valid_method(method: &str) -> bool {
    return method == "GET";
}

fn is_valid_protocol(protocol: &str) -> bool {
    if protocol == "HTTP" {
        return true;
    }

    let protocol_tokens:Vec<&str> = protocol.split('/')
        .collect();

    if protocol_tokens.len() != 2 {
        return false;
    }

    let (protocol_name, version) = (protocol_tokens[0], protocol_tokens[1]);
    if let Ok(version_number) = version.parse::<f64>() {
        return protocol_name == "HTTP" && version_number >= 0.9
    }

    false
}

fn read_file(file_path: &Path) -> Result<String, ErrorKind> {
    match File::open(file_path) {
        Ok(mut file) => {
            let mut buffer = String::new();
            file.read_to_string(&mut buffer).ok();
            Ok(buffer)
        },
        Err(err) => Err(err.kind()),
    }
}
fn create_success_response(content_type: &String, content_length: usize, data: &String) -> Response {
        Response {
            status: "200".to_string(),
            web_server: "agf453-agl475-web-server/0.1".to_string(),
            content_type: content_type.to_owned(),
            content_length: content_length,
            data: data.to_owned(),
        }
}

fn create_error_response(status: &str) -> Response {
    let res = match status {
        "400" => Response {
            status: "400".to_string(),
            web_server: "".to_string(),
            content_type: "".to_string(),
            content_length: 0,
            data: "<h1>400 Bad Request</h1>".to_string(),
        },
        "403" => Response {
            status: "403".to_string(),
            web_server: "".to_string(),
            content_type: "".to_string(),
            content_length: 0,
            data: "<h1>403 Forbidden</h1>".to_string(),
        },
        "404" => Response {
            status: "404".to_string(),
            web_server: "".to_string(),
            content_type: "".to_string(),
            content_length: 0,
            data: "<h1>404 Not Found</h1>".to_string(),
        },
        _ => panic!("Invalid error code")
    };

    res
}

fn send_response(stream: &mut TcpStream, res: &Response) {
    let r = res.clone();
    let mut output = String::new();

    if r.status == "200" {
        output = format!("HTTP/1.0 {} OK\n{}\ntext/{}\n{}\n\n{}", r.status, r.web_server, r.content_type, r.content_length, r.data);
    }
    else if r.status == "400" {
        output = format!("HTTP/1.0 {} Bad Request\n{}\n\n{}", r.status, r.web_server, r.data);
    }
    else if r.status == "403" {
        output = format!("HTTP/1.0 {} Forbidden\n{}\n\n{}", r.status, r.web_server, r.data);
    }
    else if r.status == "404" {
        output = format!("HTTP/1.0 {} Not Found\n{}\n\n{}", r.status, r.web_server, r.data);
    }
    stream.write_all(&output.as_bytes()).expect("Sending HTTP response failed.");
}

fn log_request(log_file: &Arc<Mutex<File>>, req: &Request, res: &Response) {
    let mut guard = log_file.lock().unwrap();
    let timestamp = UTC::now();

    let buffer = format!("{} {} {}\n{}\n{}\n\n", req.method, req.path, req.protocol, timestamp.to_string(), res.status);
    guard.write(&buffer.as_bytes()).expect("Log update failed.");
}


#[cfg(test)]
mod server_tests {
    use super::*;
    
    #[test]
    fn parse_empty_request_gives_error_test() {
        assert_eq!(parse_request("").is_ok(), false);
    }
    
    #[test]
    fn parse_invalid_request_gives_error_test() {
        assert_eq!(parse_request("POST /some/url HTTP 2.0").is_ok(), false);
    }

    #[test]
    fn parse_request_returns_tokens_test() {
        let expected = Request {
            method: "GET".to_string(),
            path: "/some/url".to_string(),
            protocol: "HTTP/2.0".to_string(),
        };
        assert_eq!(parse_request("GET /some/url HTTP/2.0").is_ok(), true);
        assert_eq!(parse_request("GET /some/url HTTP/2.0").unwrap(), expected);
    }

    #[test]
    fn get_is_valid_method_test() {
        assert_eq!(is_valid_method("GET"), true);
    }

    #[test]
    fn post_is_not_valid_method_test() {
        assert_eq!(is_valid_method("POST"), false);
    }

    #[test]
    fn http_is_valid_protocol_test() {
        assert_eq!(is_valid_protocol("HTTP"), true);
    }

    #[test]
    fn gibberish_is_not_valid_protocol_test() {
        assert_eq!(is_valid_protocol("alskds/llk"), false);
    }

    #[test]
    fn newer_http_version_is_valid_protocol_test() {
        assert_eq!(is_valid_protocol("HTTP/1.0"), true);
    }

    #[test]
    fn older_http_version_is_not_valid_protocol_test() {
        assert_eq!(is_valid_protocol("HTTP/0.8"), false);
    }

    #[test]
    fn handle_request_bad_request_is_400_test() {
        let req = Request {
            method: "POST".to_string(),
            path: "/lol.txt".to_string(),
            protocol: "HTTP".to_string(),
        };

        let res = handle_request(&req);
        assert_eq!(res.status, "400");
    }

    #[test]
    fn handle_request_forbidden_is_403_test() {
        // this test uses forbidden.txt, which I created with:
        // touch forbidden.txt
        // sudo chown root forbidden.txt
        // sudo chmod 700 forbidden.txt
        //
        // I'm not sure how permissions work when zipping files and sending them through canvas
        let req = Request {
            method: "GET".to_string(),
            path: "/forbidden.txt".to_string(),
            protocol: "HTTP/2.0".to_string(),
        };

        let res = handle_request(&req);
        assert_eq!(res.status, "403");
    }

    #[test]
    fn handle_request_not_found_is_404() {
        let req = Request {
            method: "GET".to_string(),
            path: "/path/does/not/exist".to_string(),
            protocol: "HTTP/2.0".to_string(),
        };

        let res = handle_request(&req);
        assert_eq!(res.status, "404");
    }

    #[test]
    fn handle_request_dir_without_index_file_is_404() {
        let req = Request {
            method: "GET".to_string(),
            path: "/src".to_string(),
            protocol: "HTTP/2.0".to_string(),
        };

        let res = handle_request(&req);
        assert_eq!(res.status, "404");
    }

    #[test]
    fn handle_request_file_is_200() {
        let req = Request {
            method: "GET".to_string(),
            path: "/lol.txt".to_string(),
            protocol: "HTTP/2.0".to_string(),
        };

        let res = handle_request(&req);
        assert_eq!(res.status, "200");
    }

    #[test]
    fn handle_request_dir_with_index_file_is_200() {
        let req = Request {
            method: "GET".to_string(),
            path: "/".to_string(),
            protocol: "HTTP/2.0".to_string(),
        };

        let res = handle_request(&req);
        assert_eq!(res.status, "200");
    }

    #[test]
    fn handle_request_content_type_test() {
        let req1 = Request {
            method: "GET".to_string(),
            path: "/index.html".to_string(),
            protocol: "HTTP/2.0".to_string(),
        };

        let req2 = Request {
            method: "GET".to_string(),
            path: "/lol.txt".to_string(),
            protocol: "HTTP/2.0".to_string(),
        };
        let res1 = handle_request(&req1);
        let res2 = handle_request(&req2);

        assert_eq!(res1.content_type, "html");
        assert_eq!(res2.content_type, "plain");
    }
}
