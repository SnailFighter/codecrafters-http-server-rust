// Uncomment this block to pass the first stage
use std::net::{TcpListener, TcpStream};
use std::io::{ErrorKind, Read, Write};
use std::{fs, thread};
use std::fs::File;
use std::env;

static mut CONFIG: Vec<EnvParam> = Vec::new();

struct Request {
    method          :   String,
    http_version    :   String,
    ip              :   String,
    port            :   u32,
    path            :   String,
    user_agent      :   String,
}

struct EnvParam {
    name            :   String,
    value           :   String,
}

impl EnvParam {
    fn new() -> EnvParam {
        EnvParam{ name: "".to_string(), value: "".to_string() }
    }
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

/*
  format: --command command-value
  for example:  --directory <directory>
  but if lost the value, we still allow it continue to run, such as
  --command1 --command2 command-value2.
  it losts the command-value1
  or  command-value1 --command2 command-value2
  it losts the first command name
 */
fn parse_env_params() ->Vec<EnvParam>{
    let params: Vec<String> = env::args().collect();
    println!("{:?}", params);
    if params.len()<1 {
        return vec![];
    }
    let mut all_param = Vec::<EnvParam>::new();
    let mut is_command_name = false;
    let mut iter = params.iter();
    for (i,x) in iter.enumerate() {
        if x.starts_with("--") {
            let mut env_param = EnvParam::new();
            env_param.name = x.to_string();
            all_param.push(env_param);
            is_command_name = true;
        }else {
            let mut env_param = all_param.pop().unwrap_or_else(|| {
                EnvParam::new()
            });
            env_param.value = x.to_string();
            all_param.push(env_param);
            is_command_name = false;
        }
    }
    all_param
}

fn read_file(file_path:String) -> Result<String,i32> {
    if std::path::Path::new(file_path.as_str()).exists(){
        let mut content = String::new();
        File::open(file_path.clone()).unwrap().read_to_string(&mut content).expect("some error!");
        Ok(content)
    }else {
        println!("the file path {} doesn't exist", file_path);
        Err(404)
    }
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

                user_agent = item.split_once(":").unwrap().1.trim().to_string();
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
    }else {
        req.path = path.trim().to_string();
    }
    req
}
fn dispatch(req: Request, stream: TcpStream) {
    let path = req.path;
    let mut resp_content = "".to_string();
    if path == "/" {
        resp_content =  "HTTP/1.1 200 \r\n\r\n".to_string();
    } else if path.starts_with("/echo/") {
        let val = path.strip_prefix("/echo/").unwrap();
        resp_content = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\r\n",
            val.len(),
            val
        );
    }else if path.starts_with("/user-agent") {
        resp_content = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\r\n",
            req.user_agent.len(),
            req.user_agent
        );
    }else if path.starts_with("/files"){
        let path_param : Vec<_>= path.split("/").collect();
        let file_name = if path_param.len()>1 {
            path_param[2]
        }else { "" };
        unsafe {
            for e in CONFIG.iter() {
                if e.name == "--directory"  {
                    if !e.value.is_empty(){
                        let content = read_file(e.value.clone()+file_name);
                        match content {
                            Ok(c) => {
                                resp_content = format!(
                                    "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}\r\n",
                                    c.len(),
                                    c
                                );
                            }
                            Err(e) => {
                                eprintln!("{}", e.to_string());
                                resp_content = "HTTP/1.1 404  Not Found\r\n\r\n".to_string()
                            }
                        }

                    }else {
                        resp_content = "HTTP/1.1 404  Not Found\r\n\r\n".to_string()
                    };

                };

            };
        };

    }else {
        resp_content = "HTTP/1.1 404  Not Found\r\n\r\n".to_string();
    };
    println!("{}", resp_content);
    response(stream, resp_content.as_str());
}

fn response(mut stream:TcpStream, resp:&str){
    stream.write_all(resp.as_bytes()).expect("Response to client failed!");
    stream.flush().expect("Some errors occurs when flush");
}

fn main(){
    unsafe { CONFIG = parse_env_params(); }

    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");
    // Uncomment this block to pass the first stage
     let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
     //listener.set_nonblocking(true).unwrap();
     //dispatch
    for stream in listener.incoming() {
         match stream {
            Ok(mut _stream) => {
                thread::spawn(move ||{
                    let req = parse_request_header(&_stream);
                    let req1 = pre_handle_path(req.path.clone(), req);
                    dispatch(req1, _stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
         }
    }
}


