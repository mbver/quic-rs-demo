use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use std::{
  path::Path,
  fs,
};
use anyhow::{Context, Result};

const CERT_DIR: &str = "/tmp/quinn_certs";

fn main() -> Result<()>{
  let cert_dir = Path::new(CERT_DIR);
  if cert_dir.exists() {
    fs::remove_dir_all(cert_dir).context("failed to clear cert_dir")?;
  }
  fs::create_dir_all(cert_dir).context("failed to create cert_dir")?;

  let cert_path = cert_dir.join("cert.der");
  let key_path = cert_dir.join("key.der");

  let cert_key = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
  let priv_key = PrivatePkcs8KeyDer::from(cert_key.key_pair.serialize_der());
  let cert: CertificateDer<'_> = cert_key.cert.into();
  
  fs::write(&cert_path, &cert).context("failed to write certificate")?;
  fs::write(&key_path, &priv_key.secret_pkcs8_der()).context("failed to write private key")?;
  println!("✅ Finished generating key!\n  📄 Cert: {}\n  🔑 Key:  {}", cert_path.display(), key_path.display());

  Ok(())
}