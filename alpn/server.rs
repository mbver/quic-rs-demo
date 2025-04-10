use rustls::{
  pki_types::{CertificateDer, PrivateKeyDer},
  crypto::{CryptoProvider, aws_lc_rs},
};
use core::ascii;
use std::{
  fmt::Write, fs, net::SocketAddr, path::Path, sync::Arc, str,
};
use anyhow::{Context, Result, bail};
use quinn::{
  crypto::rustls::QuicServerConfig,
  Endpoint,
  ServerConfig,
  SendStream, 
  RecvStream,
};
use colored::*;
// use  proto::crypto::rustls::QuicServerConfig,

const CERT_DIR: &str = "/tmp/quinn_certs";
const SERVER_ALPNS: &[&[u8]] = &[b"h1", b"h2"];

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

  let mut tls_config = rustls::ServerConfig::builder()
  .with_no_client_auth()
  .with_single_cert(vec![cert], key)?;
  tls_config.alpn_protocols = SERVER_ALPNS.iter().map(|&x| x.into()).collect();

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
  loop {
    let stream = conn.accept_bi().await;
    let stream = match stream {
      Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
        println!("connection closed");
        return Ok(());
    }
      Err(e) => {
        return Err(e.into());
      }
      Ok(s) => s
    };
    tokio::spawn(async move {
      handle_stream(stream).await
    });
  }
}

async fn handle_stream((mut send, mut recv):(SendStream, RecvStream)) -> Result<()> {
  let req = recv
  .read_to_end(64*1024)
  .await
  .context("failed reading request")?;

  let mut escaped = String::new();
  for &x in &req {
    for c in ascii::escape_default(x) {
      escaped.write_char(c as char).unwrap();
    }
  }
  println!("req {}", escaped);

  let resp = handle_req(&req).unwrap_or_else(
    |e| {
      println!("handle request failed: {}", e);
      String::from("failed to handle request").into_bytes()
  });
  send.write_all(&resp).await.context("failed to send response")?;
  send.finish().unwrap();
  println!("complete stream handling!");
  Ok(())
}

fn handle_req(req: &[u8]) -> Result<Vec<u8>> {
  // only accept GET request
  if req.len() < 4 || &req[0..4] != b"GET " {
    bail!("missing GET");
  }
  if req[4..].len() < 2 || &req[req.len() - 2..] != b"\r\n" {
      bail!("missing \\r\\n");
  }
  let filename = &req[4..req.len()-2];
  let filename = str::from_utf8(&filename).context("filename is malformed UTF-8")?;
  let path = Path::new(file!());
  let path = path.parent().unwrap().join(filename);
  let bytes = fs::read(&path).context("failed reading file")?;
  Ok(bytes)
}