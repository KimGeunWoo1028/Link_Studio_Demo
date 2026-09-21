use std::net::{IpAddr, Ipv4Addr};

use rand::Rng;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LanKind {
    Wifi,
    Ethernet,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanInterface {
    pub id: String,
    pub name: String,
    pub kind: LanKind,
    pub ipv4: String,
    pub prefix_len: Option<u8>,
    pub gateway: Option<String>,
    pub physical: bool,
    pub up: bool,
    pub selectable: bool,
    pub recommended: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AdapterSnapshot {
    pub friendly_name: String,
    pub description: String,
    pub ipv4: Ipv4Addr,
    pub prefix_len: u8,
    pub gateway: Option<Ipv4Addr>,
    pub kind: LanKind,
    pub if_type_label: &'static str,
    pub up: bool,
    #[allow(dead_code)]
    pub ipv4_metric: u32,
}

pub fn is_virtual_adapter(friendly_name: &str, description: &str) -> bool {
    let hay = format!("{friendly_name} {description}").to_ascii_lowercase();
    const MARKERS: &[&str] = &[
        "wsl",
        "hyper-v",
        "vethernet",
        "virtual ethernet",
        "docker",
        "vbox",
        "virtualbox",
        "vmware",
        "hyperv",
        "tap-windows",
        "tap adapter",
        "vpn",
        "wireguard",
        "nordlynx",
        "cisco anyconnect",
        "bluetooth",
        "kernel debug",
        "pseudo-interface",
        "loopback",
    ];
    MARKERS.iter().any(|marker| hay.contains(marker))
}

pub fn classify_if_type(if_type_label: &str) -> LanKind {
    match if_type_label {
        "ieee80211" | "wifi" | "wireless" => LanKind::Wifi,
        "ethernet" | "ethernetcsmacd" => LanKind::Ethernet,
        _ => LanKind::Other,
    }
}

pub fn skip_if_type(if_type_label: &str) -> bool {
    matches!(
        if_type_label,
        "softwareloopback" | "loopback" | "tunnel" | "ppp" | "atm" | "ieee1394"
    )
}

pub fn classify_snapshot(snap: &AdapterSnapshot) -> LanInterface {
    let virtual_nic = is_virtual_adapter(&snap.friendly_name, &snap.description)
        || skip_if_type(snap.if_type_label);
    let link_local = snap.ipv4.is_link_local() || snap.ipv4.is_loopback() || snap.ipv4.is_unspecified();
    let physical = !virtual_nic && matches!(snap.kind, LanKind::Wifi | LanKind::Ethernet);
    let mut reason = None;
    if link_local {
        reason = Some("Link-local or loopback address".into());
    } else if virtual_nic {
        reason = Some("Virtual or tunnel adapter".into());
    } else if !snap.up {
        reason = Some("Disconnected — address may be stale".into());
    }
    let selectable = snap.up && physical && !link_local && reason.is_none();
    LanInterface {
        id: format!("{}|{}", snap.friendly_name, snap.ipv4),
        name: snap.friendly_name.clone(),
        kind: snap.kind,
        ipv4: snap.ipv4.to_string(),
        prefix_len: Some(snap.prefix_len),
        gateway: snap.gateway.map(|ip| ip.to_string()),
        physical,
        up: snap.up,
        selectable,
        recommended: false,
        reason,
    }
}

pub fn mark_recommended(mut interfaces: Vec<LanInterface>, metrics: &[u32]) -> Vec<LanInterface> {
    let mut best: Option<(usize, u32)> = None;
    for (index, iface) in interfaces.iter().enumerate() {
        if !iface.selectable {
            continue;
        }
        let metric = metrics.get(index).copied().unwrap_or(u32::MAX);
        match best {
            None => best = Some((index, metric)),
            Some((_, best_metric)) if metric < best_metric => best = Some((index, metric)),
            Some((best_index, best_metric)) if metric == best_metric => {
                // Tie: do not prefer Wi-Fi over Ethernet or the reverse.
                if interfaces[index].ipv4 < interfaces[best_index].ipv4 {
                    best = Some((index, metric));
                }
            }
            _ => {}
        }
    }
    if let Some((index, _)) = best {
        interfaces[index].recommended = true;
    }
    interfaces
}

pub fn selectable_ips(interfaces: &[LanInterface]) -> Vec<IpAddr> {
    interfaces
        .iter()
        .filter(|iface| iface.selectable)
        .filter_map(|iface| iface.ipv4.parse().ok())
        .collect()
}

pub fn recommended_ipv4(interfaces: &[LanInterface]) -> Option<String> {
    interfaces
        .iter()
        .find(|iface| iface.recommended)
        .map(|iface| iface.ipv4.clone())
}

pub fn list_lan_interfaces() -> Vec<LanInterface> {
    #[cfg(windows)]
    {
        windows_nics()
    }
    #[cfg(not(windows))]
    {
        fallback_nics()
    }
}

#[cfg(not(windows))]
fn fallback_nics() -> Vec<LanInterface> {
    let snaps = local_ip_address::list_afinet_netifas()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(name, ip)| {
            let IpAddr::V4(v4) = ip else {
                return None;
            };
            Some(AdapterSnapshot {
                friendly_name: name.clone(),
                description: name,
                ipv4: v4,
                prefix_len: 24,
                gateway: None,
                kind: LanKind::Other,
                if_type_label: "other",
                up: true,
                ipv4_metric: 100,
            })
        })
        .collect::<Vec<_>>();
    let metrics = snaps.iter().map(|snap| snap.ipv4_metric).collect::<Vec<_>>();
    let classified = snaps.iter().map(classify_snapshot).collect();
    mark_recommended(classified, &metrics)
}

#[cfg(windows)]
fn windows_nics() -> Vec<LanInterface> {
    let Ok(adapters) = ipconfig::get_adapters() else {
        return Vec::new();
    };
    let mut snaps = Vec::new();
    let mut metrics = Vec::new();
    for adapter in adapters {
        let if_type_label = match adapter.if_type() {
            ipconfig::IfType::Ieee80211 => "ieee80211",
            ipconfig::IfType::EthernetCsmacd => "ethernet",
            ipconfig::IfType::SoftwareLoopback => "softwareloopback",
            ipconfig::IfType::Tunnel => "tunnel",
            ipconfig::IfType::Ppp => "ppp",
            ipconfig::IfType::Atm => "atm",
            ipconfig::IfType::Ieee1394 => "ieee1394",
            _ => "other",
        };
        let kind = classify_if_type(if_type_label);
        let up = adapter.oper_status() == ipconfig::OperStatus::IfOperStatusUp;
        let gateway = adapter.gateways().iter().find_map(|ip| match ip {
            IpAddr::V4(v4) => Some(*v4),
            _ => None,
        });
        for ip in adapter.ip_addresses() {
            let IpAddr::V4(v4) = ip else {
                continue;
            };
            let prefix_len = adapter
                .prefixes()
                .iter()
                .find_map(|(net, len)| match net {
                    IpAddr::V4(_) if *len <= 32 => Some(*len as u8),
                    _ => None,
                })
                .unwrap_or(24);
            snaps.push(AdapterSnapshot {
                friendly_name: adapter.friendly_name().to_string(),
                description: adapter.description().to_string(),
                ipv4: *v4,
                prefix_len,
                gateway,
                kind,
                if_type_label,
                up,
                ipv4_metric: adapter.ipv4_metric(),
            });
            metrics.push(adapter.ipv4_metric());
        }
    }
    let classified = snaps.iter().map(classify_snapshot).collect();
    mark_recommended(classified, &metrics)
}

pub fn lan_ipv4s() -> Vec<IpAddr> {
    selectable_ips(&list_lan_interfaces())
}

pub fn primary_lan_ipv4() -> Option<IpAddr> {
    recommended_ipv4(&list_lan_interfaces()).and_then(|ip| ip.parse().ok())
}

pub fn new_session_id() -> String {
    let mut bytes = [0u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub fn install_lan_inbound_rules() -> Result<String, String> {
    #[cfg(windows)]
    {
        windows_firewall::install()
    }
    #[cfg(not(windows))]
    {
        Err("Firewall rules are only implemented on Windows.".into())
    }
}

#[cfg(windows)]
mod windows_firewall {
    use std::process::Command;

    fn rule_exists(name: &str) -> bool {
        Command::new("netsh")
            .args(["advfirewall", "firewall", "show", "rule", &format!("name={name}")])
            .output()
            .ok()
            .map(|out| out.status.success() && String::from_utf8_lossy(&out.stdout).contains(name))
            .unwrap_or(false)
    }

    fn add_rule(name: &str, port: u16) -> Result<(), String> {
        if rule_exists(name) {
            return Ok(());
        }
        let output = Command::new("netsh")
            .args([
                "advfirewall",
                "firewall",
                "add",
                "rule",
                &format!("name={name}"),
                "dir=in",
                "action=allow",
                "protocol=TCP",
                &format!("localport={port}"),
                "profile=private,domain",
                "enable=yes",
            ])
            .output()
            .map_err(|err| err.to_string())?;
        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "{} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
            .trim()
            .to_string())
        }
    }

    pub fn install() -> Result<String, String> {
        add_rule("Link Studio LAN HTTP 8787", 8787)?;
        add_rule("Link Studio LAN HTTPS 8443", 8443)?;
        Ok("Added inbound allow rules for TCP 8787 and 8443 on Private/Domain profiles.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(name: &str, desc: &str, ip: [u8; 4], kind: LanKind, up: bool, metric: u32) -> AdapterSnapshot {
        AdapterSnapshot {
            friendly_name: name.into(),
            description: desc.into(),
            ipv4: Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]),
            prefix_len: 24,
            gateway: Some(Ipv4Addr::new(192, 168, 0, 1)),
            kind,
            if_type_label: match kind {
                LanKind::Wifi => "ieee80211",
                LanKind::Ethernet => "ethernet",
                LanKind::Other => "other",
            },
            up,
            ipv4_metric: metric,
        }
    }

    #[test]
    fn filters_wsl_docker_and_link_local() {
        let wsl = classify_snapshot(&snap(
            "vEthernet (WSL (Hyper-V firewall))",
            "Hyper-V Virtual Ethernet Adapter",
            [192, 168, 192, 1],
            LanKind::Ethernet,
            true,
            1,
        ));
        assert!(!wsl.selectable);
        let apipa = classify_snapshot(&AdapterSnapshot {
            gateway: None,
            ipv4: Ipv4Addr::new(169, 254, 59, 173),
            ..snap("Wi-Fi 4", "Intel Wi-Fi", [169, 254, 59, 173], LanKind::Wifi, false, 25)
        });
        assert!(!apipa.selectable);
    }

    #[test]
    fn does_not_select_disconnected_ethernet_stale_ip() {
        let wifi = classify_snapshot(&snap("Wi-Fi", "Intel Wi-Fi", [192, 168, 0, 137], LanKind::Wifi, true, 30));
        let ethernet = classify_snapshot(&snap(
            "Ethernet",
            "Realtek PCIe GbE Family Controller",
            [192, 168, 0, 160],
            LanKind::Ethernet,
            false,
            25,
        ));
        assert!(wifi.selectable);
        assert!(!ethernet.selectable);
        assert!(ethernet.reason.as_deref().unwrap().contains("Disconnected"));
        let marked = mark_recommended(vec![ethernet, wifi], &[25, 30]);
        assert_eq!(recommended_ipv4(&marked).as_deref(), Some("192.168.0.137"));
    }

    #[test]
    fn does_not_prefer_wifi_over_ethernet_when_both_are_up() {
        let wifi = classify_snapshot(&snap("Wi-Fi", "Intel Wi-Fi", [192, 168, 0, 137], LanKind::Wifi, true, 30));
        let ethernet = classify_snapshot(&snap(
            "Ethernet",
            "Realtek PCIe GbE Family Controller",
            [192, 168, 0, 160],
            LanKind::Ethernet,
            true,
            25,
        ));
        let marked = mark_recommended(vec![wifi, ethernet], &[30, 25]);
        assert_eq!(recommended_ipv4(&marked).as_deref(), Some("192.168.0.160"));
        let flipped = mark_recommended(
            vec![
                classify_snapshot(&snap("Wi-Fi", "Intel Wi-Fi", [192, 168, 0, 137], LanKind::Wifi, true, 20)),
                classify_snapshot(&snap(
                    "Ethernet",
                    "Realtek PCIe GbE Family Controller",
                    [192, 168, 0, 160],
                    LanKind::Ethernet,
                    true,
                    25,
                )),
            ],
            &[20, 25],
        );
        assert_eq!(recommended_ipv4(&flipped).as_deref(), Some("192.168.0.137"));
    }
}
