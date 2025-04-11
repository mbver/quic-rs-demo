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
use common::{Login, Session, ADMIN_USERNAME};
use sha2::{Sha256, Digest};
use base64::{prelude::BASE64_STANDARD, Engine};

use rand::{rngs::OsRng, TryRngCore};
use hmac::{Hmac, Mac};
mod common;
// use  proto::crypto::rustls::QuicServerConfig,

const CERT_DIR: &str = "/tmp/quinn_certs";
const ADMIN_PWD_HASH: &str = "bUUlwqIfm+HMqeQfOqQC4HZe5fzD5/6jShabFzCuOG4=";
const AUTH_BEARER_HEADER: &[u8] = b"Authentication Bearer ";
type HmacSha256 = Hmac<Sha256>;

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
  let mut key: [u8; 32] = [0u8; 32];
  conn
    .export_keying_material(&mut key, b"token-binding", b"")
    .expect("failed to export keying material");

  loop {
    let stream = conn.accept_bi().await;
    let (mut send, mut recv) = match stream {
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
      handle_stream(&mut send, &mut recv, &key).await
    });
  }
}

struct ErrClientFinished;

impl std::fmt::Debug for ErrClientFinished {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("ClientFinished: client sent FIN")
    }
}

impl std::fmt::Display for ErrClientFinished {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      write!(f, "client sent FIN")
  }
}
impl std::error::Error for ErrClientFinished {}

async fn read_line(recv: &mut RecvStream) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    let mut byte = [0u8; 1];

    while recv.read(&mut byte).await? == Some(1) {
        buf.push(byte[0]);
        if buf.ends_with(b"\r\n") {
            buf.truncate(buf.len() - 2);
            break;
        }
    }
    Ok(buf)
}

fn hash_pwd(pwd: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(pwd.as_bytes());
  let checksum = hasher.finalize();
  BASE64_STANDARD.encode(checksum)
}

fn sign_token(key: &[u8], token: &String) -> Result<String> {
  let mut m = HmacSha256::new_from_slice(key).expect("failed to create hmac from key");
  m.update(token.as_bytes());
  Ok(BASE64_STANDARD.encode(m.finalize().into_bytes()))
}

fn gen_session(key: &[u8]) -> Result<Vec<u8>> {
  let mut token = [0u8, 32];
  OsRng.try_fill_bytes(&mut token).expect("failed to generate session token");

  let token = BASE64_STANDARD.encode(token);
  let signature = sign_token(key, &token).expect("failed to sign token");

  let session = Session{token, signature};
  let json_bytes = serde_json::to_vec(&session)?;
  Ok(json_bytes)
} 



async fn auth(
  send: &mut SendStream, 
  recv: &mut RecvStream,
  key: &[u8],
) -> Result<()> {
  let mut req = read_line(recv).await.context("failed reading request")?;
  if !req.starts_with(b"POST ") {
    bail!("missing POST ");
  }
  req.drain(0..5);

  if req != b"/login" {
    bail!("missing /login");
  }

  let content_length = read_line(recv).await.context("failed reading content-length")?;
  let content_length = str::from_utf8(&content_length)?.parse::<usize>()?;
  let mut body = vec![0; content_length];
  recv.read_exact(&mut body).await.context("failed reading body")?;

  let login: Login = serde_json::from_slice(&body).context("failed to deserialize login")?;
  if login.username != ADMIN_USERNAME && hash_pwd(&login.password) != ADMIN_PWD_HASH {
    bail!("wrong username or password");
  }
  
  let session_bytes = gen_session(key)?;
  send.write_all(&session_bytes).await.context("failed to send session bytes")?;
  println!("🪪 Session established and token sent to client.");
  Ok(())
}


async fn verify_session(recv: &mut RecvStream, key: &[u8]) -> Result<()> {
  let mut line = read_line(recv).await.context("failed reading session data")?;
  if line.len() == 0 {
    return Err(ErrClientFinished{}.into());
  }
  if !line.starts_with(AUTH_BEARER_HEADER) {
    bail!("missing Authentication Bearer ");
  }
  line.drain(0..AUTH_BEARER_HEADER.len());
  let session: Session = serde_json::from_slice(&line)?;
  let sig = sign_token(key, &session.token)?;
  if sig != session.signature {
    bail!("wrong signature");
  }
  Ok(())
}

async fn handle_stream(send: &mut SendStream, recv: &mut RecvStream, key: &[u8]) -> Result<()> {
  auth(send, recv, key).await.context("authentication failed")?;

  loop {
    match verify_session(recv, key).await.context("verify session failed") {
        Ok(_) => {},
        Err(e) if e.downcast_ref::<ErrClientFinished>().is_some() => {
          println!("client has finished (sent FIN)");
          break;
        }
        Err(e) => {
          println!("verify session failed: {:?}", e);
          return Err(e);
        }
    };
    let mut req = read_line(recv).await.context("failed reading request")?;
    let mut escaped = String::new();
    for &x in &req {
      for c in ascii::escape_default(x) {
        escaped.write_char(c as char).unwrap();
      }
    }
    println!("req {}", escaped);
  
    let resp = handle_req(&mut req).unwrap_or_else(
      |e| {
        println!("handle request failed: {}", e);
        String::from("failed to handle request").into_bytes()
    });
    send.write_all(&resp).await.context("failed to send response")?;
  }
  send.finish().unwrap();
  println!("complete stream handling!");
  Ok(())
}


fn handle_req(req: &mut Vec<u8>) -> Result<Vec<u8>> {
  // only accept GET request
  if !req.starts_with(b"GET ") {
    bail!("missing GET ");
  }
  req.drain(0..4);

  let filename = str::from_utf8(req).context("filename is malformed UTF-8")?;
  let path = Path::new(file!());
  let path = path.parent().unwrap().join(filename);
  let bytes = fs::read(&path).context("failed reading file")?;
  Ok(bytes)
}