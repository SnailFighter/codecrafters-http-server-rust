// Uncomment this block to pass the first stage
use std::net::TcpListener;
use std::io::Write;

fn main(){
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
     let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    //
    for stream in listener.incoming() {
         match stream {
            Ok(mut _stream) => {
                 println!("accepted new connection");
                _stream.write("HTTP/1.1 200 \r\n\r\n".as_bytes()).expect("Response to client failed!");
                _stream.flush().expect("Some errors occurs when flush");
            }
            Err(e) => {
                println!("error: {}", e);
         }
       }
     }
}
