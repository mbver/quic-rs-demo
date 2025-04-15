use std::{
  fs, net::SocketAddr, path::Path, str, sync::Arc
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
  println!("sending hello datagram...");
  conn.send_datagram(b"Hello from client"[..].into()).context("failed sending datagram")?;
  let msg = conn.read_datagram().await.context("failed to receive datagram response")?;
  println!("received response: \n{}", std::str::from_utf8(&msg)?);
  println!("terminating connection...");
  conn.close(0u32.into(), b"done");
  println!("Done!");
  Ok(())
}