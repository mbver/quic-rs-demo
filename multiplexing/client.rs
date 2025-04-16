use std::{
  fs, io::{self, Write}, net::SocketAddr, path::Path, sync::Arc, time::Duration,
};
use anyhow::{Context, Result};
use rustls::{
  pki_types::CertificateDer,
  crypto::{CryptoProvider, aws_lc_rs},
};

use quinn::{
  Connection,
  crypto::rustls::QuicClientConfig,
  Endpoint,
  ClientConfig,
};
const CERT_DIR: &str = "/tmp/quinn_certs";

#[tokio::main]
async fn main() -> Result<()> {
  let cert_dir: &Path = Path::new(CERT_DIR);
  let cert_path= cert_dir.join("cert.der");
  let mut cert_root = rustls::RootCertStore::empty();
  cert_root.add(CertificateDer::from(fs::read(cert_path)?))?;

  CryptoProvider::install_default(
    aws_lc_rs::default_provider()
  ).expect("failed to install default crypto provider");

  let tls_config = rustls::ClientConfig::builder()
    .with_root_certificates(cert_root)
    .with_no_client_auth();
  let quic_config = QuicClientConfig::try_from(tls_config)?;
  let client_config = ClientConfig::new(Arc::new(quic_config));
  
  let addr: SocketAddr = "127.0.0.1:4385".parse()?;
  let mut endpoint = Endpoint::client(addr)?;
  endpoint.set_default_client_config(client_config);

  let server_addr = "127.0.0.1:4843".parse()?;
  let conn = endpoint
    .connect(server_addr,"localhost" )?
    .await
    .context("failed to connect to server")?;
  
  println!("connected to server {}", server_addr.to_string());

  tokio::try_join!(
    get_sample(&conn, 0),
    get_sample(&conn, 1),
    upload_data(&conn),
    send_datagram(&conn),
  )?;

  println!("\nclosing connection...");
  // gracefully close the connection
  conn.close(0u32.into(), b"done");
  tokio::time::sleep(Duration::from_millis(100)).await;
  println!("done");
  Ok(())
}

async fn get_sample(conn: &Connection, stream_no: u32) -> Result<()> {
  println!("\nopen bidirectional stream number {}", stream_no);
  let (mut send, mut recv) = conn
  .open_bi()
  .await
  .context("failed to open bi_stream")?;
  println!("stream number {} is sending request ...", stream_no);
  let req = format!("GET {}\r\n", "sample.json");
  send.write_all(req.as_bytes())
    .await
    .context("failed to send request")?;
  send.finish().unwrap();

  println!("stream number {} is receiving response ...", stream_no);
  let resp = recv
    .read_to_end(usize::MAX)
    .await
    .context("failed to read response")?;

  println!("\nresponse received on stream number {}:", stream_no);

  io::stdout().write_all(&resp).unwrap();
  io::stdout().flush().unwrap();
  println!("");
  Ok(())
}

async fn upload_data(conn: &Connection) -> Result<()> {
  println!("\nopen unidirectional stream");
  let mut send = conn
  .open_uni()
  .await
  .context("failed to open uni_stream")?;

  let path = Path::new(file!());
  let path = path.parent().unwrap().join("sample.json");
  let data = fs::read(&path).context("failed reading file")?;
  println!("uni_stream uploading data...");
  send.write_all(&data)
    .await
    .context("failed to send request")?;
  send.finish().unwrap();
  println!("Done uploading data with uni_stream!");
  Ok(())
}

async fn send_datagram(conn: &Connection) -> Result<()> {
  println!("\nStart sending/receiving datagram...");
  conn.send_datagram(b"Hello from client"[..].into()).context("failed to send datagram")?;
  let msg = conn.read_datagram().await.context("failed to receive datagram response")?;
  println!("recevied datagram response: \n{}", std::str::from_utf8(&msg)?);
  println!("Done sending/receiving datagram!");
  Ok(())
}