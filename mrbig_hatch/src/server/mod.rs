// ############################################################################
// #                                                                          #
// # mrbig_hatch/src/server/mod.rs                                            #
// #                                                                          #
// # Handcrafted with love by MrBig Mobsters                                  #
// # All rights reserved                                                      #
// #                                                                          #
// #                                                                          #
// # Description: Hatch server structure and implementation.                  #
// ############################################################################

//! Hatch server
//!

// Temporarily allow unused
#![allow(dead_code, unused)]

// Import external dependencies
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::stream::StreamExt;

// Import local dependencies
use crate::errors::Result;
use std::env;


/// Hatch server structure
pub struct HatchServer {

  /// TCP port the hatch server is listening to
  port: u32,

  /// Internet socket address (IPv4 or IPv6) the hatch is serving on.
  address: String,
}

// Public functions
impl HatchServer {

  /// Builds a new hatch server instance
  pub fn new() -> Self {
    HatchServer {
      port: 9999,
      address: "127.0.0.1".to_string(),
      //Some(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9999))
    }
  }

  // /// Starts the asynchronous hatch server
  // pub async fn run(self) -> std::result::Result<(), Box<dyn std::error::Error>> {

  //   let address = "127.0.0.1:9999";

  //   // Create a TCP listener which will listen for incoming connections.
  //   let mut listener = TcpListener::bind(&address).await?;
  //   println!("Hatch server is listening on: {}", address);

  //   // Here we convert the `TcpListener` to a stream of incoming connections
  //   // with the `incoming` method. We then define how to process each element in
  //   // the stream with the `for_each` combinator method
  //   let server = async move {
  //     let mut incoming = listener.incoming();
  //     while let Some(socket_res) = incoming.next().await {
  //         match socket_res {
  //             Ok(socket) => {
  //                 println!("Accepted connection from {:?}", socket.peer_addr());
  //                 // TODO: Process socket
  //             }
  //             Err(err) => {
  //               // Handle error by printing to STDOUT.
  //               println!("accept error = {:?}", err);
  //             }
  //         }
  //     }
  //   };

  //   println!("Hatch server running on {}", address );

  //   // Start the server and block this async fn until `server` spins down.
  //   server.await;
  // }

    // loop {
    //   // Wait for an inbound socket asynchronously.
    //   match server.accept().await {

    //     Ok((socket, _)) => {

    //       // Handle incoming requests concurrently (i.e. each request in its own non-blocking thread)
    //       tokio::spawn( async move {

    //         // We're parsing each socket with the `BytesCodec` included in `tokio::codec`.
    //         let mut framed = BytesCodec::new().framed(socket);

    //         // We loop while there are messages coming from the Stream `framed`.
    //         // The stream will return None once the client disconnects.
    //         while let Some(message) = framed.next().await {
    //             match message {
    //                 Ok(bytes) => println!("bytes: {:?}", bytes),
    //                 Err(err) => println!("Socket closed with error: {:?}", err),
    //             }
    //         }
    //         println!("Socket received FIN packet and closed connection");
    //       });
    //     }

    //     Err(error) => println!("error accepting a socket; error is {:?}", error),
    //   }
    // }


  pub async fn run(self) -> Result<()> { //std::result::Result<(), Box<dyn std::error::Error>> {

    // Allow passing an address to listen on as the first argument of this
    // program, but otherwise we'll just set up our TCP listener on
    // 127.0.0.1:8080 for connections.
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    // Next up we create a TCP listener which will listen for incoming
    // connections. This TCP listener is bound to the address we determined
    // above and must be associated with an event loop.
    let mut listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    loop {
        // Asynchronously wait for an inbound socket.
        let (mut socket, _) = listener.accept().await?;

        // Essentially here we're executing a new task to run concurrently,
        // which will allow all of our clients to be processed concurrently.
        tokio::spawn(async move {
            let mut buf = [0; 1024];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = socket
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                if n == 0 {
                    return;
                }

                socket
                    .write_all(&buf[0..n])
                    .await
                    .expect("failed to write data to socket");
            }
        });
    }
  }

  pub async fn start(self) -> Result<()>  {


    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let mut listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    let server = async move {
      let mut incoming = listener.incoming();

      while let Some(connection) = incoming.next().await {
          match connection {
              Ok(socket) => {
                  println!("Accepted connection from {:?}", socket.peer_addr());
                  // TODO: Process socket
              }

              Err(err) => {
                // Handle error by printing to STDOUT.
                println!("accept error = {:?}", err);
              }
          }
      }
    };

    println!("Hatch server running on {}", addr );

    // Start the server and block this async fn until `server` spins down.
    server.await;

    Ok(())
  }

} // end of 'impl HatchServer'


// Private helpers
