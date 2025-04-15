use rustls::{
  pki_types::{CertificateDer, PrivateKeyDer},
  crypto::{CryptoProvider, aws_lc_rs},
};
use std::{
  fs, net::SocketAddr, path::Path, sync::Arc, str,
};
use anyhow::{Context, Result};
use quinn::{
  crypto::rustls::QuicServerConfig,
  Endpoint,
  ServerConfig,
};
use colored::*;
// use  proto::crypto::rustls::QuicServerConfig,

const CERT_DIR: &str = "/tmp/quinn_certs";

#[tokio::main]
async fn main() -> Result<()> {
  let cert_dir: &Path = Path::new(CERT_DIR);
  let cert_path= cert_dir.join("cert.der");
  let key_path = cert_dir.join("key.der");

  let bytes = fs::read(cert_path).context("failed to read certificate")?;
  let cert = CertificateDer::try_from(bytes)?;

  let bytes = fs::read(key_path).context("failed to read private key")?;
  let key =  PrivateKeyDer::try_from(bytes).map_err(anyhow::Error::msg)?;

  CryptoProvider::install_default(
    aws_lc_rs::default_provider()
  ).expect("failed to install default crypto provider");

  let tls_config = rustls::ServerConfig::builder()
  .with_no_client_auth()
  .with_single_cert(vec![cert], key)?;

  let quic_config = QuicServerConfig::try_from(tls_config)?;
  let server_config = ServerConfig::with_crypto(Arc::new(quic_config));

  let addr: SocketAddr = "127.0.0.1:4843".parse()?;
  let endpoint = Endpoint::server(server_config, addr)?;

  let addr = endpoint.local_addr()?;
  println!(
    "{} {}",
    "🚀 QUIC server listening at:".bold().green(),
    addr.to_string().blue());

  while let Some(conn) = endpoint.accept().await {
    println!("accepting connection from {}", conn.remote_address());
    tokio::spawn(async move {
      handle_conn(conn).await
    });  
  }
  Ok(())
}

async fn handle_conn(incomming: quinn::Incoming) -> Result<()> {
  let conn = incomming.await?;
  println!("established connection from {}", conn.remote_address());
  let msg = conn.read_datagram().await.context("failed to read datagram")?;
  println!("receive msg: \n{}", std::str::from_utf8(&msg)?);
  println!("responding to client {}...", conn.remote_address());
  conn.send_datagram(b"Hello from server"[..].into()).context("failed to send response")?;
  // wait for client termination
  loop {
    match conn.read_datagram().await {
        Ok(msg) => {
            println!("received: {}", std::str::from_utf8(&msg)?);
            conn.send_datagram(b"ack"[..].into())?;
        }
        Err(quinn::ConnectionError::ApplicationClosed {..}) => {
          println!("client {} terminated", conn.remote_address());
          break
        },
        Err(e) => return Err(e.into()),
    }
  }
  println!("Done handle conn from {}", conn.remote_address());
  Ok(())
}
