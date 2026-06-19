use anyhow::Result;
use local_ip_address::list_afinet_netifas;
use std::net::IpAddr;

pub fn get_local_interfaces() -> Result<Vec<String>> {
    let interfaces = list_afinet_netifas()?;
    let mut ips = Vec::new();
    for (_name, ip) in interfaces {
        if let IpAddr::V4(ipv4) = ip {
            if !ipv4.is_loopback() {
                ips.push(ipv4.to_string());
            }
        }
    }
    ips.sort();
    ips.dedup();
    Ok(ips)
}
