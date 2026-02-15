use rcgen::{CertificateParams, KeyPair};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let params = CertificateParams::new(["localhost".to_string()])?;
    let signing_key = KeyPair::generate_for(&rcgen::PKCS_RSA_SHA512)?;
    let cert = params.self_signed(&signing_key)?;

    print!("writing certificate to cert.pem ... ");
    fs::write("cert.pem", cert.pem())?;
    println!("DONE");

    print!("writing key to key.pem ... ");
    fs::write("key.pem", signing_key.serialize_pem())?;
    println!("DONE");

    Ok(())
}
