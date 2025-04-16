use std::{
  fs, io::{self, Write}, net::SocketAddr, path::Path, sync::Arc,
};
use anyhow::{Context, Result};
use rustls::{
  pki_types::{CertificateDer,PrivateKeyDer},
  crypto::{CryptoProvider, aws_lc_rs},
};

use quinn::{
  crypto::rustls::QuicClientConfig,
  Endpoint,
  ClientConfig,
};
const SERVER_CERT_PATH: &str = "/tmp/quinn_certs/server_cert.der";
const CLIENT_CERT_PATH: &str = "/tmp/quinn_certs/client_cert.der";
const CLIENT_KEY_PATH: &str = "/tmp/quinn_certs/client_key.der";

#[tokio::main]
async fn main() -> Result<()> {
  // setup authenticated client
  let server_cert_path = Path::new(SERVER_CERT_PATH);
  let mut remote_cert_root = rustls::RootCertStore::empty();
  remote_cert_root.add(CertificateDer::from(fs::read(server_cert_path)?))?;

  let cert_path = Path::new(CLIENT_CERT_PATH);
  let key_path = Path::new(CLIENT_KEY_PATH);

  let bytes: Vec<u8> = fs::read(cert_path).context("failed to read certificate")?;
  let cert = CertificateDer::try_from(bytes)?;

  let bytes = fs::read(key_path).context("failed to read private key")?;
  let key =  PrivateKeyDer::try_from(bytes).map_err(anyhow::Error::msg)?;

  CryptoProvider::install_default(
    aws_lc_rs::default_provider()
  ).expect("failed to install default crypto provider");

  let tls_config = rustls::ClientConfig::builder()
    .with_root_certificates(remote_cert_root.clone())
    .with_client_auth_cert(vec![cert], key)?;

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

  // anonymous client will not be authenticated
  let tls_config = rustls::ClientConfig::builder()
    .with_root_certificates(remote_cert_root)
    .with_no_client_auth();

  let quic_config = QuicClientConfig::try_from(tls_config)?;
  let client_config = ClientConfig::new(Arc::new(quic_config));

  let addr: SocketAddr = "127.0.0.1:4386".parse()?;
  let mut endpoint = Endpoint::client(addr)?;
  endpoint.set_default_client_config(client_config);

  println!("\n\nanonymous client connecting...");
  let server_addr = "127.0.0.1:4843".parse()?;
  let conn = endpoint
    .connect(server_addr,"localhost" )?
    .await
    .context("failed to connect to server")?;

  println!("connected to server {}", server_addr.to_string());

  println!("anonymous client opening stream...");
  let (mut send, mut recv) = conn
    .open_bi()
    .await
    .context("failed to open stream")?;
  println!("done opening stream!");

  println!("anonymous client sending request...");
  let req = format!("GET {}\r\n", "sample.json");
    send.write_all(req.as_bytes())
      .await
      .context("failed to send request")?;
    send.finish().unwrap();
  println!("done sending request!");

  println!("anonymous client reading response...");
  let result = recv
      .read_to_end(usize::MAX)
      .await;

  match result {
    Ok(_) => {
      // We expected this to fail, so success is an error.
      eprintln!("❌ Unexpected success: expected the connection to fail due to missing certificate.");
      std::process::exit(1); // or return Err(...) if inside a function
    }
    Err(e) => {
      println!("✅ Expected Error: {:?}", e);
    }
  }
  conn.close(0u32.into(), b"done");
  Ok(())
}