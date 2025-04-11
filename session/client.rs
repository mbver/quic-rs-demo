use std::{
  fs, io::{self, Write}, net::SocketAddr, path::Path, sync::Arc
};
use anyhow::{Context, Result};
use rustls::{
  pki_types::CertificateDer,
  crypto::{CryptoProvider, aws_lc_rs},
};

use quinn::{
  crypto::rustls::QuicClientConfig,
  Endpoint,
  ClientConfig,
};
const CERT_DIR: &str = "/tmp/quinn_certs";
const ADMIN_PWD: &str = "admin_password";

mod common;

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

  let (mut send, mut recv) = conn
  .open_bi()
  .await
  .context("failed to open stream")?;

  let req = format!("GET {}\r\n", "sample.json");
  send.write_all(req.as_bytes())
    .await
    .context("failed to send request")?;
  send.finish().unwrap();

  let resp = recv
    .read_to_end(usize::MAX)
    .await
    .context("failed to read response")?;

  println!("response received:");

  io::stdout().write_all(&resp).unwrap();
  io::stdout().flush().unwrap();
  conn.close(0u32.into(), b"done");

  Ok(())
}