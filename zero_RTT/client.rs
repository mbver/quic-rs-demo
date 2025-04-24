use std::{
  fs, io::{self, Write}, net::{SocketAddr, UdpSocket}, path::Path, sync::Arc, time::Duration,
};
use anyhow::{Context, Result};
use rustls::pki_types::CertificateDer;

use quinn::{
  EndpointConfig,
  TokioRuntime,
  Endpoint,
  Connection,
  ClientConfig,
};
const CERT_DIR: &str = "/tmp/quinn_certs";

#[tokio::main]
async fn main() -> Result<()> {
  let endpoint = endpoint();
  let server_addr = "127.0.0.1:4843".parse()?;

  println!("initial connection...");
  let conn = endpoint
  .connect(server_addr, "localhost")
  .unwrap()
  .into_0rtt()
  .err()
  .expect("0-RTT should not succeed without established connection to resume")
  .await
  .expect("connect");

  println!("connected to server {}", server_addr.to_string());

  get_sample(&conn).await.context("failed to get sample.json")?;
  println!("posting something in full handshake...");
  post_something(&conn).await.context("failed to post something")?;
  drop(conn);

  println!("\nresuming connection...");

  let (conn, _zero_rtt) = endpoint
  .connect(server_addr, "localhost")
  .unwrap()
  .into_0rtt()
  .unwrap_or_else(|_| panic!("resuming connection with 0-RTT failed"));
  
  println!("0-RTT connected server {}", server_addr.to_string());
  get_sample(&conn).await.context("failed to get sample.json")?;
  println!("resending request...");
  get_sample(&conn).await.context("failed to get sample.json")?;
  println!("posting something after 0-rtt...");
  post_something(&conn).await.context("failed to post something after 0-RTT")?;
  drop(conn);

  println!("\nresuming connection again...");
  let (conn, _zero_rtt) = endpoint
  .connect(server_addr, "localhost")
  .unwrap()
  .into_0rtt()
  .unwrap_or_else(|_| panic!("resuming connection with 0-RTT failed"));
  
  println!("0-RTT connected server {}", server_addr.to_string());
  println!("posting something in 0-rtt...");
  post_something(&conn).await?;
  drop(conn);

  println!("\nresuming connection for replay attack...");
  let (conn, _zero_rtt) = endpoint
  .connect(server_addr, "localhost")
  .unwrap()
  .into_0rtt()
  .unwrap_or_else(|_| panic!("resuming connection with 0-RTT failed"));
  
  println!("0-RTT connected server {}", server_addr.to_string());
  println!("replay requests in 0-rtt...");
  replay_attack(&conn).await?;
  Ok(())
}

async fn get_sample(conn: &Connection) -> Result<()> {
  let (mut send, mut recv) = conn
  .open_bi()
  .await
  .context("failed to open bi_stream")?;
  let req = format!("GET {}\r\n", "sample.json");
  send.write_all(req.as_bytes())
    .await
    .context("failed to send request")?;
  send.finish().unwrap();

  let resp = recv
    .read_to_end(usize::MAX)
    .await
    .context("failed to read response")?;
  io::stdout().write_all(&resp).unwrap();
  io::stdout().flush().unwrap();
  println!("");
  Ok(())
}

async fn post_something(conn: &Connection) -> Result<()> {
  let (mut send, mut recv) = conn
  .open_bi()
  .await
  .context("failed to open bi_stream")?;
  let req = format!("POST /something {}\r\n", "some important thing");
  send.write_all(req.as_bytes())
    .await
    .context("failed to send request")?;
  send.finish().unwrap();

  let resp = recv
    .read_to_end(usize::MAX)
    .await
    .context("failed to read response")?;
  io::stdout().write_all(&resp).unwrap();
  io::stdout().flush().unwrap();
  println!("");
  Ok(())
}

async fn replay_attack(conn: &Connection) -> Result<()> {
  for _ in 0..5 {
    let c = conn.clone();
    tokio::spawn(async move {
      get_sample(&c).await
    });
  }
  tokio::time::sleep(Duration::from_millis(100)).await;
  Ok(())
}

// TODO: 0RTT only works if we setup server and client endpoint with Endpoint::new
// instead of Endpoint::server and Endpoint::client. WHY?
fn endpoint() -> Endpoint {
  let cert_dir: &Path = Path::new(CERT_DIR);
  let cert_path= cert_dir.join("cert.der");

  let bytes = fs::read(cert_path).context("failed to read certificate").unwrap();
  let cert = CertificateDer::try_from(bytes).unwrap();

  let mut roots = rustls::RootCertStore::empty();
  roots.add(cert.clone()).unwrap();

  let client_config = ClientConfig::with_root_certificates(Arc::new(roots)).unwrap();

  let addr: SocketAddr = "127.0.0.1:4385".parse().unwrap();
  let mut endpoint = Endpoint::new(
    EndpointConfig::default(),
    None,
    UdpSocket::bind(addr).unwrap(),
    Arc::new(TokioRuntime),
  ).unwrap();

  endpoint.set_default_client_config(client_config);

  endpoint
}