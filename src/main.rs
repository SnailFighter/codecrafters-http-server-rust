// Uncomment this block to pass the first stage
use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::{fs, thread};
use std::fs::File;
use std::env;
use std::ops::Add;
use std::path::Path;
use nom::character::complete::i64;

static mut CONFIG: Vec<EnvParam> = Vec::new();

struct Header {
    method          :   String,
    http_version    :   String,
    ip              :   String,
    port            :   u32,
    path            :   String,
    user_agent      :   String,
}

struct Body<T> {
    content            :   T,
}


struct Request<T> {
    header          :   Header,
    body            :   Body<T>,
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
        println!("{},{}, {}, {}, {},{}", buf[len-1], buf[len-2], buf[len-3], buf[len-4], b'\n', b'd');
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
        File::open(file_path.clone()).unwrap().read_to_string(&mut content).unwrap_or_else(|_|404);
        Ok(content)
    }else {
        println!("the file path {} doesn't exist", file_path);
        Err(404)
    }
}

fn parse_request(mut stream: &TcpStream) ->Request<String> {
    let get   ="get";
    let post  = "post";
    let mut ip:         String = "".to_string();
    let mut port:       u32 = 80;
    let mut method:     String = "".to_string();
    let mut http_version:    String = "".to_string();
    let mut path:           String = "".to_string();
    let mut user_agent:     String ="".to_string();
    let mut content_length  :   i32 = 0;

    let mut body        :String = "".to_string();

    println!("accepted new connection");
    let mut data = vec![0u8,0];
    let mut buffer = [0; 1024*1024];
    stream.read(&mut buffer).unwrap();
    let mut request = String::from_utf8(buffer.to_vec()).unwrap();
    /*loop {
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
    };*/
    //let request = String::from_utf8(data).unwrap();
    /*let mut reader = BufReader::new(stream);
    let mut request =  String::new();

    loop{
        reader.read_line(&mut request).unwrap();
        if request.ends_with("\r\n\r\n") {
            break;
        }
    }*/

    let v: Vec<&str> = request.lines().map(|line|line).collect();
    let mut is_start_body = false;
    for line in v {
        if !line.is_empty() && !is_start_body {
            if line.to_lowercase().contains(get) {
                method = get.to_string().to_uppercase();
            }else if line.to_lowercase().contains(post) {
                method = post.to_string().to_uppercase();
            }
            if line.contains("HTTP") {
                let version = line.split_whitespace().nth(2);
                http_version = version.unwrap().to_string();
                path = line.split_whitespace().nth(1).unwrap_or("/").to_string();
                continue;
            }
            if line.contains("Host") {
                ip = line.split(":").nth(1).unwrap().to_string();
                let port_str = line.split(":").nth(2).unwrap();
                port = port_str.parse().unwrap_or(0);
                continue;
            }
            if line.contains("User-Agent") {
                user_agent = line.split_once(":").unwrap().1.trim().to_string();
                continue;
            }
            if line.contains("Content-Length") {
                let content_length_str = line.split_once(":").unwrap().1.trim();
                content_length = content_length_str.parse::<i32>().unwrap();
            }

        }else if  line.is_empty() && !is_start_body {
            is_start_body = true;
        }
    }

    /*loop{
        if body.len() as i32 == content_length {
            break;
        }
        reader.read_line(&mut body).unwrap();
    }*/


    let header = Header {
        method,
        http_version,
        ip,
        port,
        path,
        user_agent,
    };

    Request {
        header,
        
        body: Body { content:body },
    }

}

fn pre_handle_path(mut path: String, mut req: Request<String>) -> Request<String> {
    if path.is_empty() {
        req.header.path = "/".to_string();
    }else {
        req.header.path = path.trim().to_string();
    }
    req
}
unsafe fn dispatch(req: Request<String>, stream: TcpStream) {
    let path = req.header.path;
    let mut resp_content = "HTTP/1.1 200\r\n\r\n".to_string();
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
            req.header.user_agent.len(),
            req.header.user_agent
        );
    }else if path.starts_with("/files"){
        if req.header.method == "Get" {
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
        }else if req.header.method == "POST"{
            let mut path_param: Vec<_>= path.split("/").collect();
            let file_name = if path_param.len()>1 {
                path_param[2]
            }else { "unname" };
            for e in CONFIG.iter() {
                if e.name == "--directory"  {
                    if !e.value.is_empty(){
                        let mut file = File::create(Path::new(format!("{}/{}", e.value, file_name).as_str())).expect("TODO: panic message");
                        file.write_all(req.body.content.as_bytes()).unwrap();
                        resp_content = "HTTP/1.1 201 \r\n\r\n".to_string()
                    }else {
                        resp_content = "HTTP/1.1 404  Not Found\r\n\r\n".to_string()
                    };

                };
            }

        }

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
                thread::spawn(move || unsafe {
                    let req = parse_request(&_stream);
                    let req1 = pre_handle_path(req.header.path.clone(), req);
                    dispatch(req1, _stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
         }
    }
}


