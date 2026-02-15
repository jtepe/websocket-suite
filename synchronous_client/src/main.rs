use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;

use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, ServerName};
use rustls::{ClientConfig, ClientConnection, RootCertStore};
use tungstenite::Message;
use tungstenite::protocol::WebSocket;
use url::{Host, Url};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wss_url = std::env::var("WSS_URL").unwrap_or_else(|_| {
        format!(
            "wss://{}:{}",
            std::env::var("WS_HOST").unwrap(),
            std::env::var("WS_PORT").unwrap()
        )
    });
    let url: Url = wss_url.parse()?;

    let cert_path = std::env::var("CERT_PATH").unwrap();
    println!("adding server certificate to truststore ...");
    let cert =
        CertificateDer::from_pem_file(cert_path).expect("certificate file cert.pem in PEM format");
    let mut roots = RootCertStore::empty();
    roots.add(cert)?;

    let client_config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();

    let server_name: ServerName<'_> = match url.host().expect("hostname is valid").to_owned() {
        Host::Domain(s) => s
            .try_into()
            .expect("hostname can be converted to server name"),
        Host::Ipv4(s) => s.into(),
        Host::Ipv6(s) => s.into(),
    };

    let host_tuple = format!("{}:{}", server_name.to_str(), url.port().unwrap_or(3043));

    let connect_retries: usize = std::env::var("CONNECT_RETRIES").unwrap().parse()?;
    let connect_delay = std::time::Duration::from_millis(
        std::env::var("CONNECT_DELAY_MS").unwrap().parse::<u64>()?,
    );

    let mut n = 0;
    let mut sock = loop {
        n += 1;
        if n > connect_retries {
            return Err(format!("failed to connect after {connect_retries} attempts").into());
        }
        match TcpStream::connect(&host_tuple) {
            Ok(sock) => break sock,
            Err(e) if matches!(e.kind(), std::io::ErrorKind::ConnectionRefused) => {
                std::thread::sleep(connect_delay);
                continue;
            }
            Err(e) => panic!("failed to connect: {e}"),
        }
    };

    let mut conn = ClientConnection::new(Arc::new(client_config), server_name)
        .expect("client connection created");
    let stream = rustls::Stream::new(&mut conn, &mut sock);

    let (mut websocket, _response) = tungstenite::client(url, stream).unwrap();

    println!("successfully connected to server at {host_tuple}",);

    println!("now sending ping ...");
    websocket.send(Message::Ping(b"client ping"[..].into()))?;
    receive(&mut websocket)?;

    println!("now sending text ...");
    websocket.send(Message::Text("how are you doing?".into()))?;
    receive(&mut websocket)?;

    println!("now sending close ... expecting close from server ...");
    websocket.close(None)?;
    websocket.flush()?;

    let msg = websocket.read().expect("close message from server");
    if msg.is_close() {
        eprintln!("close received from server");
    } else {
        eprintln!("error: expected close but received {:?} from server", msg);
    }

    match websocket.read().expect_err("connection close error") {
        tungstenite::Error::ConnectionClosed => eprintln!("server closed connection cleanly"),
        err => eprintln!("error: expected close from server: received {err}"),
    }

    Ok(())
}

fn receive<Stream>(websocket: &mut WebSocket<Stream>) -> tungstenite::Result<()>
where
    Stream: Read + Write,
{
    match websocket.read()? {
        Message::Binary(_) | Message::Frame(_) => {
            unreachable!();
        }
        Message::Ping(bytes) => {
            println!("received ping with content {bytes:?}");
        }
        Message::Pong(bytes) => {
            println!("received pong with content {bytes:?}");
        }
        Message::Text(utf8) => {
            println!("received text message: {utf8}");
        }
        Message::Close(_) => {
            unimplemented!("should not be handled by this function");
        }
    }
    Ok(())
}
