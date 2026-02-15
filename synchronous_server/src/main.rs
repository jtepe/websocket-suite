use rustls::pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
use std::net::TcpListener;
use std::sync::Arc;
use tungstenite::{Message, accept};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cert_path = std::env::var("CERT_PATH").unwrap();
    let key_path = std::env::var("KEY_PATH").unwrap();

    println!("adding server certificate to certstore ...");
    let cert_chain = CertificateDer::pem_file_iter(cert_path)
        .expect("certificate file in valid PEM format")
        .map(|cert| cert.expect("valid PEM certificate"))
        .collect();
    println!("add server private key file ...");
    let key = PrivateKeyDer::from_pem_file(key_path).expect("private key file in valid PEM format");

    let server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key)
        .expect("TLS server config from certificate and key file");

    let port = std::env::var("WS_PORT").unwrap().parse::<u16>()?;
    let bind_host = std::env::var("WS_BIND").unwrap();
    let host_port = format!("{bind_host}:{port}");
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
