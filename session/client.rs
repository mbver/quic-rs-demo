use std::{
  fs, io::{self, Write}, net::SocketAddr, path::Path, sync::Arc
};
use anyhow::{Context, Result};
use common::{Login, ADMIN_USERNAME, Session};
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
  let login= Login{
    username: ADMIN_USERNAME.to_string(), 
    password: ADMIN_PWD.to_string(),
  };
  let login_str = serde_json::to_string(&login).unwrap();
  let content_length = login_str.len();

  let req: String = format!("POST /login\r\n{}\r\n{}", content_length, login_str);
  send.write_all(req.as_bytes())
    .await
    .context("failed to send request")?;

  let mut buf = [0u8; 1024];
  let n: usize = recv
                          .read(&mut buf)
                          .await?
                          .expect("failed reading session cookie");
  let session: Session = serde_json::from_slice(&buf[0..n]).context("failed to deserialize session")?;
  let session_str = serde_json::to_string(&session)?;
  println!("✅ Login success. Session token received");
  let req = format!("Authentication Bearer {}\r\nGET {}\r\n", session_str, "sample.json");
  for i in 0..3 {
    println!("\nsending request number {}...", i);
    send.write_all(req.as_bytes())
      .await
      .context("failed to send request")?;
    let n: usize = recv
    .read(&mut buf)
    .await?
    .expect("failed reading response");
    println!("response received:");

    io::stdout().write_all(&buf[0..n]).unwrap();
    io::stdout().flush().unwrap();
    println!();
  }
  send.finish().unwrap();
  conn.close(0u32.into(), b"done");
  Ok(())
}

