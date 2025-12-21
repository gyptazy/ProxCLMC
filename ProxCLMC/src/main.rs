/*
 * ProxCLMC - Proxmox CPU Live Migration Checker
 *
 * Copyright (C) 2025 Florian Paul Azim Hoberg @gyptazy <gyptazy@gyptazy.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::fs;
use std::io;
use std::io::Read;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::time::Duration;
use clap::Parser;
use ssh2::Session;

#[derive(Debug)]
struct Node {
    name: String,
    ring0_addr: String,
    cpu: String,
    cpu_type: String,
}

#[derive(Parser, Debug)]
#[command(name = "proxclmc")]
#[command(
    about = "ProxCLMC - Proxmox CPU Live Migration Checker",
    long_about = None
)]
struct Cli {
    #[arg(short, long = "ssh-file", default_value = "/root/.ssh/id_rsa")]
    ssh_file: PathBuf,

    #[arg(long)]
    version: bool,

    #[arg(short, long)]
    verbose: bool,
}

fn parse_corosync_conf<P: AsRef<Path>>(path: P) -> io::Result<Vec<Node>> {
    let content = fs::read_to_string(path)?;
    let mut nodes = Vec::new();
    let mut in_nodelist = false;
    let mut in_node = false;
    let mut current_name: Option<String> = None;
    let mut current_ip: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();

        match line {
            "nodelist {" => in_nodelist = true,

            "node {" if in_nodelist => {
                in_node = true;
                current_name = None;
                current_ip = None;
            }

            "}" if in_node => {
                if let (Some(name), Some(ip)) = (current_name.take(), current_ip.take()) {
                    nodes.push(Node {
                        name,
                        ring0_addr: ip,
                        cpu: String::new(),
                        cpu_type: String::new(),
                    });
                }
                in_node = false;
            }

            "}" if in_nodelist => in_nodelist = false,

            _ if in_node => {
                if let Some(v) = line.strip_prefix("name:") {
                    current_name = Some(v.trim().to_string());
                } else if let Some(v) = line.strip_prefix("ring0_addr:") {
                    current_ip = Some(v.trim().to_string());
                }
            }

            _ => {}
        }
    }

    Ok(nodes)
}

fn ssh_read_cpuinfo(
    ip: &str,
    user: &str,
    ssh_key: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    let tcp = TcpStream::connect(format!("{}:22", ip))?;
    tcp.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;
    sess.userauth_pubkey_file(user, None, ssh_key, None)?;

    if !sess.authenticated() {
        return Err("SSH authentication failed".into());
    }

    let mut channel = sess.channel_session()?;
    channel.exec("cat /proc/cpuinfo")?;
    let mut output = String::new();
    channel.read_to_string(&mut output)?;
    channel.wait_close()?;

    Ok(output)
}

fn extract_flags(cpuinfo: &str) -> Vec<String> {
    for line in cpuinfo.lines() {
        if line.starts_with("flags") {
            return line
                .split(':')
                .nth(1)
                .unwrap_or("")
                .split_whitespace()
                .map(|f| f.to_string())
                .collect();
        }
    }
    Vec::new()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CpuType {
    V1,
    V2,
    V3,
    V4,
}

impl CpuType {
    fn from_flags(flags: &[String]) -> Self {
        let has = |f: &str| flags.iter().any(|x| x == f);

        if has("avx512f") {
            Self::V4
        } else if has("avx") && has("avx2") {
            Self::V3
        } else if has("sse4_2") && has("popcnt") {
            Self::V2
        } else {
            Self::V1
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "x86-64-v1" => Some(Self::V1),
            "x86-64-v2" => Some(Self::V2),
            "x86-64-v3" => Some(Self::V3),
            "x86-64-v4" => Some(Self::V4),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::V1 => "x86-64-v1",
            Self::V2 => "x86-64-v2",
            Self::V3 => "x86-64-v3",
            Self::V4 => "x86-64-v4",
        }
    }
}

fn enrich_node_cpu_ssh(
    node: &mut Node,
    ssh_user: &str,
    ssh_key: &Path,
) {
    match ssh_read_cpuinfo(&node.ring0_addr, ssh_user, ssh_key) {
        Ok(cpuinfo) => {
            let flags = extract_flags(&cpuinfo);
            let cpu_type = CpuType::from_flags(&flags);
            node.cpu = "remote-detected".to_string();
            node.cpu_type = cpu_type.as_str().to_string();
        }
        Err(e) => {
            eprintln!(
                "Failed to detect CPU on {} ({}): {}",
                node.name, node.ring0_addr, e
            );
        }
    }
}

fn cluster_min_cpu_type(nodes: &[Node]) -> Option<CpuType> {
    nodes
        .iter()
        .map(|n| CpuType::from_str(&n.cpu_type))
        .collect::<Option<Vec<_>>>()?
        .into_iter()
        .min()
}

fn main() -> io::Result<()> {
    let args = Cli::parse();

    if args.version {
        println!("ProxCLMC");
        println!("Description: A lightweight tool to determine the maximum CPU compatibility level that is supported across all nodes in a Proxmox VE cluster.");
        println!("Version: 1.0.0");
        println!("Author: Florian Paul Azim Hoberg @gyptazy <gyptazy@gyptazy.com>");
        println!("GitHub: https://github.com/gyptazy/ProxCLMC");
        std::process::exit(0);
    }

    let corosync_path = "/etc/pve/corosync.conf";
    let mut nodes = parse_corosync_conf(corosync_path)?;

    if args.verbose {
        println!("Using SSH key: {}", args.ssh_file.display());
        println!("ProxCLMC version: {}", args.version);
    }

    for node in &mut nodes {
        enrich_node_cpu_ssh(node, "root", &args.ssh_file);
    }

    println!("Detected nodes:");
    for n in &nodes {
        println!("{} | {} | {}", n.name, n.ring0_addr, n.cpu_type);
    }

    if let Some(cluster_cpu) = cluster_min_cpu_type(&nodes) {
        println!("\nCluster CPU type: {}", cluster_cpu.as_str());
    } else {
        eprintln!("\nFailed to determine cluster CPU type");
    }

    Ok(())
}