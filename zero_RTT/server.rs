use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use core::ascii;
use std::{
  fmt::Write, fs, net::{SocketAddr, UdpSocket}, path::Path, sync::Arc, str,
};
use anyhow::{Context, Result, bail, anyhow};
use quinn::{
  EndpointConfig,
  TokioRuntime,
  Endpoint,
  ServerConfig,
  Connection,
  Incoming,
  SendStream, 
  RecvStream,
};
use colored::*;
// use  proto::crypto::rustls::QuicServerConfig,

const CERT_DIR: &str = "/tmp/quinn_certs";

#[tokio::main]
async fn main() -> Result<()> {
  let endpoint = endpoint();
  let addr = endpoint.local_addr()?;
  println!(
    "{} {}",
    "🚀 QUIC server listening at:".bold().green(),
    addr.to_string().blue());

  while let Some(incomming) = endpoint.accept().await {
    println!("accepting incomming connection from {}", incomming.remote_address());
    tokio::spawn(async move {
      handle_incomming(incomming).await
    }); 
  }
  Ok(())
}

async fn handle_incomming(incomming: Incoming) -> Result<()> {
  let connecting = incomming.accept()?;
  // into_0rtt is degraded to full handshake if 0-rtt is rejected
  // TODO: zero_rtt is always false even 0RTT is accepted. IS THIS A BUG?
  let Ok((conn, _zero_rtt))= connecting.into_0rtt() else {
    return Err(anyhow!("failed establishing connection"));
  };
  println!("established connection from {}", conn.remote_address());

  tokio::spawn(async move {
    handle_conn(conn).await
  });

  Ok(())
}

async fn handle_conn(conn: Connection) -> Result<()> {
  loop {
    let stream = conn.accept_bi().await;
    let (send, recv) = match stream {
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
      handle_stream(send, recv).await
    });
  }
}

async fn handle_stream(mut send: SendStream, mut recv: RecvStream) -> Result<()> {
  let is_0rtt = recv.is_0rtt();
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

  let resp = handle_req(&req, is_0rtt).unwrap_or_else(
    |e| {
      println!("handle request failed: {}", e);
      String::from("failed to handle request").into_bytes()
  });
  send.write_all(&resp).await.context("failed to send response")?;
  send.finish().unwrap();
  println!("complete stream handling!");
  Ok(())
}

fn handle_req(req: &[u8], is_0rtt: bool) -> Result<Vec<u8>> {
  println!("req is_0rtt {}", is_0rtt);
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

// TODO: 0RTT only works if we setup server and client endpoint with Endpoint::new
// instead of Endpoint::server and Endpoint::client. WHY?
fn endpoint() -> Endpoint {
  let cert_dir: &Path = Path::new(CERT_DIR);
  let cert_path= cert_dir.join("cert.der");
  let key_path = cert_dir.join("key.der");

  let bytes = fs::read(cert_path).context("failed to read certificate").unwrap();
  let cert = CertificateDer::try_from(bytes).unwrap();

  let bytes = fs::read(key_path).context("failed to read private key").unwrap();
  let key =  PrivateKeyDer::try_from(bytes).map_err(anyhow::Error::msg).unwrap();


  let server_config = ServerConfig::with_single_cert(
  vec![cert.clone()], key).unwrap();
  let mut roots = rustls::RootCertStore::empty();
  roots.add(cert.clone()).unwrap();

  let addr: SocketAddr = "127.0.0.1:4843".parse().unwrap();

  let endpoint = Endpoint::new(
    EndpointConfig::default(),
    Some(server_config),
    UdpSocket::bind(addr).unwrap(),
    Arc::new(TokioRuntime),
  ).unwrap();

  endpoint
}