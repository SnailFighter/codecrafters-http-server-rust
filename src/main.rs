// Uncomment this block to pass the first stage
use std::net::TcpListener;
use std::io::{ErrorKind, Read, Write};


fn if_data_read_end(buf: &Vec<u8>) ->bool {
    let len = buf.len();
    return if len <=4 {
        false
    }else {
        //because it always is the '\r\n\r\n' to end the request.
        buf[len-1]==b'\n' && buf[len-2]==b'\r' && buf[len-3]==b'\n' && buf[len-4]==b'\r'
    };
}

fn main(){
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");
    // Uncomment this block to pass the first stage
     let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
     //listener.set_nonblocking(true).unwrap();
    //
    for stream in listener.incoming() {
         match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                let mut data = vec![0u8,0];
                let mut buffer = [0; 256];
                loop {
                    if if_data_read_end(&data) {
                        break;
                    }
                    let result = _stream.read(&mut buffer);
                    match result {
                        Ok(len)=>{
                            if len==0 {
                                println!("len==0, have read all data from stream");
                                break;
                            }
                            data.append(&mut buffer[..len].to_vec());
                        },
                        Err(e)=>{
                            if e.kind() == ErrorKind::UnexpectedEof {
                                println!("EOF, have read all data from stream");
                                break;
                            }
                        }
                    }
                }
                if data.is_empty() {
                    println!("receive nothing ", );
                }else {
                    //let  content = String::from_utf8(all_datas).expect("error when converts String");
                    let request = String::from_utf8(data).unwrap();
                    println!("the request content is {}", request);

                    let path = request.split_whitespace().nth(1).unwrap();
                    println!("the request path is {}", path);
                    let response = if "/"==path {
                        "HTTP/1.1 200 \r\n\r\n"
                    }else {
                        "HTTP/1.1 404  Not Found\r\n\r\n"
                    };
                    _stream.write(response.as_bytes()).expect("Response to client failed!");
                    _stream.flush().expect("Some errors occurs when flush");
                }
            }
            Err(e) => {
                println!("error: {}", e);
         }
       }
     }
}
