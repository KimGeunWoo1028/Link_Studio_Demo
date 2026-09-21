use std::net::IpAddr;
use std::path::{Path, PathBuf};

use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType, ExtendedKeyUsagePurpose, IsCa,
    Issuer, KeyPair, KeyUsagePurpose,
};
use uuid::Uuid;
use base64::Engine;

pub struct CertBundle {
    pub ca_pem: PathBuf,
    pub cert_pem: PathBuf,
    pub key_pem: PathBuf,
}

pub fn ensure_tls(dir: &Path, dns_names: &[String], ips: &[IpAddr]) -> anyhow::Result<CertBundle> {
    std::fs::create_dir_all(dir)?;
    let ca_pem_path = dir.join("ca.pem");
    let ca_key_path = dir.join("ca-key.pem");
    let cert_pem_path = dir.join("server.pem");
    let key_pem_path = dir.join("server-key.pem");

    if !ca_pem_path.exists() || !ca_key_path.exists() {
        let mut params = CertificateParams::default();
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::CrlSign,
        ];
        params.use_authority_key_identifier_extension = true;
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "Link Studio Local CA");
        dn.push(DnType::OrganizationName, "Link Studio");
        params.distinguished_name = dn;
        let key = KeyPair::generate()?;
        let cert = params.self_signed(&key)?;
        std::fs::write(&ca_pem_path, cert.pem())?;
        std::fs::write(&ca_key_path, key.serialize_pem())?;
    }

    let ca_pem = std::fs::read_to_string(&ca_pem_path)?;
    let ca_key_pem = std::fs::read_to_string(&ca_key_path)?;
    let ca_key = KeyPair::from_pem(&ca_key_pem)?;
    let issuer = Issuer::from_ca_cert_pem(&ca_pem, ca_key)?;

    let mut san = vec!["localhost".to_string()];
    san.extend(dns_names.iter().cloned());
    for ip in ips {
        san.push(ip.to_string());
    }
    san.push("127.0.0.1".into());

    let mut params = CertificateParams::new(san)?;
    params.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyEncipherment,
    ];
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    params.use_authority_key_identifier_extension = true;
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, "Link Studio");
    params.distinguished_name = dn;

    let server_key = KeyPair::generate()?;
    let server_cert = params.signed_by(&server_key, &issuer)?;
    std::fs::write(&cert_pem_path, server_cert.pem())?;
    std::fs::write(&key_pem_path, server_key.serialize_pem())?;

    Ok(CertBundle {
        ca_pem: ca_pem_path,
        cert_pem: cert_pem_path,
        key_pem: key_pem_path,
    })
}

pub fn server_san_values(cert_pem: &str) -> anyhow::Result<Vec<String>> {
        let (_, pem) = x509_parser::pem::parse_x509_pem(cert_pem.as_bytes())
            .map_err(|err| anyhow::anyhow!(err.to_string()))?;
        let cert = pem
            .parse_x509()
            .map_err(|err| anyhow::anyhow!(err.to_string()))?;
    let mut names = Vec::new();
    if let Ok(Some(san)) = cert.subject_alternative_name() {
        for name in &san.value.general_names {
            match name {
                x509_parser::extensions::GeneralName::DNSName(value) => {
                    names.push((*value).to_string());
                }
                x509_parser::extensions::GeneralName::IPAddress(bytes) => {
                    if bytes.len() == 4 {
                        names.push(format!("{}.{}.{}.{}", bytes[0], bytes[1], bytes[2], bytes[3]));
                    }
                }
                _ => {}
            }
        }
    }
    Ok(names)
}

pub fn server_cert_covers_ips(cert_pem: &str, ips: &[IpAddr]) -> bool {
    let Ok(sans) = server_san_values(cert_pem) else {
        return false;
    };
    ips.iter().all(|ip| sans.iter().any(|san| san == &ip.to_string()))
}

pub fn ca_certificate_der(pem: &str) -> anyhow::Result<Vec<u8>> {
    let mut cursor = std::io::Cursor::new(pem.as_bytes());
    let parsed = rustls_pemfile::certs(&mut cursor)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| anyhow::anyhow!(err))?;
    let cert = parsed
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("CA PEM did not contain a certificate"))?;
    Ok(cert.as_ref().to_vec())
}

pub fn ios_root_ca_mobileconfig(ca_pem: &str) -> anyhow::Result<String> {
    let der = ca_certificate_der(ca_pem)?;
    if der.first() != Some(&0x30) {
        anyhow::bail!("CA DER is not a valid X.509 structure");
    }
    let payload_uuid = Uuid::new_v5(&Uuid::NAMESPACE_OID, &der);
    let profile_uuid = Uuid::new_v5(&Uuid::NAMESPACE_DNS, &der);
    let b64 = wrap_base64(&base64::engine::general_purpose::STANDARD.encode(&der));
    Ok(format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>PayloadContent</key>
  <array>
    <dict>
      <key>PayloadCertificateFileName</key>
      <string>link-studio-local-ca.cer</string>
      <key>PayloadContent</key>
      <data>
{b64}
      </data>
      <key>PayloadDescription</key>
      <string>Installs the Link Studio LAN development certificate authority.</string>
      <key>PayloadDisplayName</key>
      <string>Link Studio Local CA</string>
      <key>PayloadIdentifier</key>
      <string>com.linkstudio.app.localca.cert</string>
      <key>PayloadType</key>
      <string>com.apple.security.root</string>
      <key>PayloadUUID</key>
      <string>{payload_uuid}</string>
      <key>PayloadVersion</key>
      <integer>1</integer>
    </dict>
  </array>
  <key>PayloadDescription</key>
  <string>Trust Link Studio HTTPS on this Wi-Fi so the phone camera page can use getUserMedia.</string>
  <key>PayloadDisplayName</key>
  <string>Link Studio Local CA</string>
  <key>PayloadIdentifier</key>
  <string>com.linkstudio.app.localca</string>
  <key>PayloadOrganization</key>
  <string>Link Studio</string>
  <key>PayloadRemovalDisallowed</key>
  <false/>
  <key>PayloadType</key>
  <string>Configuration</string>
  <key>PayloadUUID</key>
  <string>{profile_uuid}</string>
  <key>PayloadVersion</key>
  <integer>1</integer>
</dict>
</plist>
"#
    ))
}

fn wrap_base64(value: &str) -> String {
    value
        .as_bytes()
        .chunks(64)
        .map(|chunk| String::from_utf8_lossy(chunk))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    fn sample_ca_pem() -> String {
        let dir = std::env::temp_dir().join(format!("link-studio-ca-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        ensure_tls(&dir, &[], &[IpAddr::V4(Ipv4Addr::new(192, 168, 0, 137))]).unwrap();
        let pem = std::fs::read_to_string(dir.join("ca.pem")).unwrap();
        let _ = std::fs::remove_dir_all(dir);
        pem
    }

    #[test]
    fn ca_der_is_x509_and_not_a_private_key() {
        let pem = sample_ca_pem();
        assert!(pem.contains("BEGIN CERTIFICATE"));
        assert!(!pem.contains("BEGIN PRIVATE KEY"));
        let der = ca_certificate_der(&pem).unwrap();
        assert_eq!(der[0], 0x30);
    }

    #[test]
    fn ios_mobileconfig_is_root_ca_profile_without_keys() {
        let pem = sample_ca_pem();
        let profile = ios_root_ca_mobileconfig(&pem).unwrap();
        assert!(profile.contains("com.apple.security.root"));
        assert!(profile.contains("<key>PayloadType</key>"));
        assert!(profile.contains("Configuration"));
        assert!(profile.contains("Link Studio Local CA"));
        assert!(!profile.contains("PRIVATE KEY"));
        assert!(!profile.contains("BEGIN CERTIFICATE"));
        let start = profile.find("<data>").unwrap() + 6;
        let end = profile.find("</data>").unwrap();
        let raw = profile[start..end]
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect::<String>();
        let der = base64::engine::general_purpose::STANDARD.decode(raw).unwrap();
        assert_eq!(der, ca_certificate_der(&pem).unwrap());
    }

    #[test]
    fn server_certificate_san_includes_all_lan_ips() {
        let dir = std::env::temp_dir().join(format!("link-studio-san-{}", Uuid::new_v4()));
        let wifi: IpAddr = Ipv4Addr::new(192, 168, 0, 137).into();
        let ethernet: IpAddr = Ipv4Addr::new(192, 168, 0, 160).into();
        ensure_tls(&dir, &[], &[wifi, ethernet]).unwrap();
        let pem = std::fs::read_to_string(dir.join("server.pem")).unwrap();
        let sans = server_san_values(&pem).unwrap();
        assert!(sans.contains(&"192.168.0.137".into()));
        assert!(sans.contains(&"192.168.0.160".into()));
        assert!(sans.contains(&"127.0.0.1".into()));
        assert!(server_cert_covers_ips(&pem, &[wifi, ethernet]));
        let _ = std::fs::remove_dir_all(dir);
    }
}
