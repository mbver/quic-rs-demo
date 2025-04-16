use rustls::{
  pki_types::{CertificateDer, PrivateKeyDer},
  crypto::{CryptoProvider, aws_lc_rs},
};
use core::ascii;
use std::{
  fmt::Write, fs, io::{self}, net::SocketAddr, path::Path, str, sync::Arc, time::Duration
};
use io::Write as IoWrite;
use anyhow::{Context, Result, bail};
use quinn::{
  crypto::rustls::QuicServerConfig,
  Endpoint,
  ServerConfig,
  TransportConfig,
  Incoming,
  Connection,
  SendStream, 
  RecvStream,
  ConnectionError,
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

  let mut transport_config = TransportConfig::default();
  transport_config.max_idle_timeout(Some(Duration::from_secs(2).try_into()?));

  let quic_config = QuicServerConfig::try_from(tls_config)?;
  let mut server_config = ServerConfig::with_crypto(Arc::new(quic_config));
  server_config.transport_config(Arc::new(transport_config));

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

async fn handle_conn(incomming: Incoming) -> Result<()> {
  let conn = incomming.await?;
  println!("established connection from {}", conn.remote_address());
  loop {
    tokio::select! {
      result = conn.accept_bi() => {
        match result {
          Err(e) => {
            return handle_error(e);
          }
          Ok((send, recv)) => {
            println!("accepting bidirectional stream...");
            tokio::spawn(async move {
              handle_bi_stream(send, recv).await
            });
          }
        }
      }

      result = conn.accept_uni() => {
        match result {
          Err(e) => {
            return handle_error(e);
          }
          Ok(recv) => {
            println!("accepting unidirectional stream...");
            tokio::spawn(async move {
              handle_uni_stream(recv).await
            });
          }
        }
      }

      result = conn.read_datagram() => {
        match result {
          Err(e) => {
            return handle_error(e);
          }
          Ok(msg) => {
            println!("accepting datagram from client...");
            if let Err(e) = handle_datagram(&conn, msg) {
              eprintln!("datagram error {:?}", e);
            }
          }
        }
      }
    }
  }
}

async fn handle_bi_stream(mut send: SendStream, mut recv:  RecvStream) -> Result<()> {
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
  println!("complete bidirectional stream handling!");
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

async fn handle_uni_stream(mut recv: RecvStream) -> Result<()> {
  let data = recv
    .read_to_end(64*1024)
    .await
    .context("failed reading data")?;

    println!("\ndata received from client:");

  io::stdout().write_all(&data).unwrap();
  io::stdout().flush().unwrap();
  println!("\nDone handle uni_stream!");
  Ok(())
}

fn handle_error(e: ConnectionError) -> Result<()> {
  match e {
    quinn::ConnectionError::ApplicationClosed { .. } => {
      println!("connection closed");
      Ok(())
    }
    quinn::ConnectionError::TimedOut { .. } => {
      println!("timeout waiting, drop connection");
      Ok(())
    }
    other => Err(other.into()),
  }
}

fn handle_datagram(conn: &Connection, msg: bytes::Bytes) -> Result<()> {
  let msg = std::str::from_utf8(&msg);
  match msg {
    Ok(msg) => {
      println!("received datagram: \n{}", msg);
      println!("sending datagram to client {}...", conn.remote_address());
      conn.send_datagram(b"Hello from server"[..].into()).context("failed to send datagram response")?;
      println!("Done respond to datagram!");
      Ok(())
    }
    Err(e) => {
      Err(e.into())
    }
  }
}