use rustls::pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
use std::net::TcpListener;
use std::sync::Arc;
use tungstenite::{Message, accept};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args();
    if args.len() < 2 {
        return Err("please provide at least the port to listen on".into());
    }

    println!("adding server certificate to certstore ...");
    let cert_chain = CertificateDer::pem_file_iter("cert.pem")
        .expect("certificate file cert.pem in valid PEM format")
        .map(|cert| cert.expect("valid PEM certificate"))
        .collect();
    println!("add server private key file ...");
    let key = PrivateKeyDer::from_pem_file("key.pem")
        .expect("private key file key.pem in valid PEM format");

    let server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key)
        .expect("TLS server config from certificate and key file");

    let port = args
        .skip(1)
        .next()
        .unwrap()
        .parse::<u16>()
        .expect("valid port number passed as argument");

    let host_port = format!("localhost:{port}");
    println!("starting server on wss://{host_port}");
    let listener = TcpListener::bind(host_port)?;
    let (mut tcp_stream, _) = listener.accept()?;
    let mut conn = rustls::ServerConnection::new(Arc::new(server_config))?;
    let tls_stream = rustls::Stream::new(&mut conn, &mut tcp_stream);
    let mut websocket = accept(tls_stream).expect("handshake over TLS stream");

    loop {
        let msg = websocket.read()?;
        if msg.is_close() || msg.is_empty() {
            eprintln!("received close from client: closing websocket connection");
            match websocket.close(None) {
                Err(tungstenite::Error::ConnectionClosed) => {
                    eprintln!("successfully closed connection");
                }
                Err(err) => {
                    eprintln!("unexpected err: {err}");
                }
                _ => {
                    unreachable!();
                }
            }
            break;
        } else if msg.is_binary() {
            eprintln!("received binary message. ignoring ...");
        } else if msg.is_text() {
            websocket.send(Message::Text(
                format!("client send message: {}: echoing back to client", msg).into(),
            ))?;
        } else {
            let m = if msg.is_ping() {
                Message::Pong("pong".into())
            } else {
                Message::Ping("ping".into())
            };
            websocket.send(m)?;
        }
    }

    websocket.get_mut().conn.send_close_notify();
    if websocket.get_ref().conn.wants_write() {
        let tls_stream = websocket.get_mut();
        let (conn, socket) = (&mut tls_stream.conn, &mut tls_stream.sock);
        conn.complete_io(socket)?;
    }

    Ok(())
}
