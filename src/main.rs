// Uncomment this block to pass the first stage
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use itertools::Itertools;


struct Request {
    method          :   String,
    http_version    :   String,
    ip              :   String,
    port            :   u32,
    path            :   String,
    user_agent      :   String,
}

fn if_data_read_end(buf: &Vec<u8>) ->bool {
    let len = buf.len();
    return if len <=4 {
        false
    }else {
        //because it always is the '\r\n\r\n' to end the request.
        buf[len-1]==b'\n' && buf[len-2]==b'\r' && buf[len-3]==b'\n' && buf[len-4]==b'\r'
    };
}


fn parse_request_header(mut stream: &TcpStream) ->Request {
    let get   ="get";
    let post  = "post";
    let mut ip:         String = "".to_string();
    let mut port:       u32 = 80;
    let mut method:     String = "".to_string();
    let mut http_version:    String = "".to_string();
    let mut path:           String = "".to_string();
    let mut user_agent:     String ="".to_string();

    println!("accepted new connection");
    let mut data = vec![0u8,0];
    let mut buffer = [0; 256];
    loop {
        if if_data_read_end(&data) {
            break;
        }
        let length = stream.read(&mut buffer).unwrap();
        if length==0 {
            println!("len==0, have read all data from stream");
            break;
        }else {
            data.append(&mut buffer[..length].to_vec());
        }
    };
    let request = String::from_utf8(data).unwrap();
    let v: Vec<&str> = request.lines().map(|line|line).collect();
    for item in v {
        if !item.is_empty()  {
            if item.to_lowercase().contains(get) {
                method = get.to_string().to_uppercase();
            }else if item.to_lowercase().contains(post) {
                method = post.to_string().to_uppercase();
            }
            if item.contains("HTTP") {
                let version = item.split_whitespace().nth(2);
                http_version = version.unwrap().to_string();
                path = item.split_whitespace().nth(1).unwrap_or("/").to_string();
            }
            if item.contains("Host") {
                ip = item.split(":").nth(1).unwrap().to_string();
                let port_str = item.split(":").nth(2).unwrap();
                port = port_str.parse().unwrap_or(0);
            }
            if item.contains("User-Agent") {

                user_agent = item.split_once(":").unwrap().1.to_string();


            }

        }
    }

    Request {
        method,
        http_version,
        ip,
        port,
        path,
        user_agent,
    }
}

fn pre_handle_path(mut path: String, mut req: Request) -> Request {
    if path.is_empty() {
        req.path = "/".to_string();
    } else if !path.ends_with("/") {
        path.push_str("/");
        req.path = path;
    };
    req
}
fn dispatch(req: Request, stream: TcpStream) {
    let path = req.path;
    let resp_content=
    if path == "/" {
        "HTTP/1.1 200 \r\n\r\n".to_string()
    } else if path=="/echo/" {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\r\n",
            path.len(),
            path
        )
    }else if path=="/user-agent/" {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\r\n",
            req.user_agent.len(),
            req.user_agent
        )
    }else {
         "HTTP/1.1 404  Not Found\r\n\r\n".to_string()
    };
    response(stream, resp_content.as_str());
}

fn response(mut stream:TcpStream, resp:&str){
    stream.write_all(resp.as_bytes()).expect("Response to client failed!");
    stream.flush().expect("Some errors occurs when flush");
}

fn main(){
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");
    // Uncomment this block to pass the first stage
     let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
     //listener.set_nonblocking(true).unwrap();
    //dispatch
    for stream in listener.incoming() {
         match stream {
            Ok(mut _stream) => {
                let req = parse_request_header(&_stream);
                let req1 = pre_handle_path(req.path.clone(), req);
                dispatch(req1, _stream);
            }
            Err(e) => {
                println!("error: {}", e);
         }
       }
     }
}


