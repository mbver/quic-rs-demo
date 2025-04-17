use std::{
  fs, io::{self, Write}, net::SocketAddr, path::Path, sync::Arc
};
use anyhow::{Context, Result, bail};
use rustls::{
  pki_types::CertificateDer,
  crypto::{CryptoProvider, aws_lc_rs},
};

use quinn::{
  crypto::rustls::QuicClientConfig,
  Endpoint,
  Connection,
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

  println!("initial connection...");
  let server_addr = "127.0.0.1:4843".parse()?;
  let connect = endpoint
    .connect(server_addr,"localhost" )?
    .into_0rtt();

  let conn = match connect {
    Ok(_) => {
      println!("O-RTT succeeded unexpectedly");
      bail!("unexpected 0-RTT success");
    }
    Err(fallback) => {
      println!("O-RTT unavailable, falling back to full-handshake");
      fallback.await.context("full handshake failed")?
    }
  };
  
  println!("connected to server {}", server_addr.to_string());

  get_sample(&conn).await.context("failed to get sample.json")?;
  drop(conn);


  println!("resuming connection...");
  let server_addr = "127.0.0.1:4843".parse()?;
  let connect = endpoint
    .connect(server_addr,"localhost" )?
    .into_0rtt();

  let conn = match connect {
    Ok((conn, _established)) => {
      println!("successfully resumed connection with 0RTT");
      conn
    }
    Err(fallback) => {
      println!("O-RTT unavailable, falling back to full-handshake");
      fallback.await.context("full handshake failed")?
    }
  };
  println!("connected to server {}", server_addr.to_string());

  get_sample(&conn).await.context("failed to get sample.json")?;
  drop(conn);
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