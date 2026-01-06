# ProxCLMC
<img align="center" src="https://cdn.gyptazy.com/img/ProxCLMC_logo.png"/>
<br>
<p float="center"><img src="https://img.shields.io/github/license/gyptazy/ProxCLMC"/><img src="https://img.shields.io/github/contributors/gyptazy/ProxCLMC"/><img src="https://img.shields.io/github/last-commit/gyptazy/ProxCLMC/main"/><img src="https://img.shields.io/github/issues-raw/gyptazy/ProxCLMC"/><img src="https://img.shields.io/github/issues-pr/gyptazy/ProxCLMC"/></p>

## Table of Contents
- [ProxCLMC](#proxclmc)
  - [General](#general)
  - [Why ProxCLMC?](#why-proxclmc)
  - [How it works](#how-it-works)
  - [Example Output](#example-output)
  - [Installation](#installation)
    - [Requirements / Dependencies](#requirements--dependencies)
    - [Install: Via Debian Repository](#install-via-debian-repository)
    - [Install: Via .deb Package](#install-via-deb-package)
    - [Usage (Run proxclmc)](#usage-run-proxclmc)
    - [Options](#options)
  - [Building from Source](#building-from-source)
  - [License](#license)
  - [Author](#author)


## General
**ProxCLMC** (Prox CPU Live Migration Checker) is a lightweight tool to determine the **maximum CPU compatibility level** that is supported across all nodes in a **Proxmox VE cluster**. It remotely inspects each cluster node via SSH, analyzes CPU capabilities, and calculates the **lowest common x86-64 CPU baseline** to ensure safe live migration and predictable VM behavior. By default, `proxclmc` runs on any Proxmox VE node, inspects the local `corosync.conf` and parses all nodes within a cluster to detect the maximum of supported CPU compatibility level.

## Why ProxCLMC?
In mixed-hardware Proxmox clusters, CPU feature mismatches are a common source of:
- Failed live migrations
- Unstable VM behavior
- Severe performance degradation (especially for Windows VMs when using the `host` CPU type)

ProxCLMC provides a **deterministic, cluster-wide answer** to the question:

> Which CPU type can I safely use for all VMs in this cluster?

## How it works
- Parses cluster nodes directly from `corosync.conf`
- Connects to each node via SSH
- Reads `/proc/cpuinfo` remotely
- Extracts CPU flags
- Maps flags to x86-64 CPU baselines
- Calculates the **lowest common CPU type** across all nodes
- Fully compatible with Proxmox / QEMU CPU models:
  - `x86-64-v1`
  - `x86-64-v2-AES`
  - `x86-64-v3`
  - `x86-64-v4`

This brings in the CPU compatibility validation to Proxmox VE based clusters, which might already be known from other virtualiaztion solutions (e.g., VMware EVC). 

## Example Output
All available nodes in a Proxmox Cluster will be listed by their supported CPU model. Afterwards, the maximum usable CPU type for the cluster will be printed that can be used for VMs to ensure safe live-migrations:
```
test-pmx01 | 10.10.10.21 | x86-64-v3
test-pmx02 | 10.10.10.22 | x86-64-v3

Cluster CPU type: x86-64-v3
```

For pipeline integration it can also simply be called with `--list-only` argument to return only the desired CPU type:
```
$> proxclmc --list-only
x86-64-v3
```

## Installation
### Requirements / Dependencies
* Proxmox VE Cluster
* SSH Authentication between all PVE nodes

### Install: Via Debian Repository
The packaged version for Debian based systems (including Proxmox) are shipped within [@gyptazy](https://github.com/gyptazy)'s Debian repository (which also serves the ProxLB project). The repository can simply be added by running the following commands and installing the Debian package `proxclmc`:
```bash
echo "deb https://repo.gyptazy.com/stable /" > /etc/apt/sources.list.d/proxlb.list
wget -O /etc/apt/trusted.gpg.d/proxlb.asc https://repo.gyptazy.com/repository.gpg
apt-get update && apt-get -y install proxclmc
```

### Install: Via .deb Package
As an alternative, you can also simply download the Debian package from [@gyptazy](https://github.com/gyptazy)'s CDN and install it afterwards:
```
wget https://cdn.gyptazy.com/debian/proxclmc/proxclmc_1.2.0_amd64.deb
dpkg -i proxclmc_1.2.0_amd64.deb
```

### Usage (Run proxclmc)
After installing the Debian package via the Debian repository or from the gyptazy CDN, you can simply run the command:
```bash
proxclmc
```

### Options
The following cli options are available:

| Short | Long          | Argument     | Default                 | Description        |
|------:|---------------|--------------|--------------------------|--------------------|
| `-s`  | `--ssh-file`. | `<SSH_FILE>` | `/root/.ssh/id_rsa`      | SSH private key to use |
|       | `--version`   |              |                          | Print version information |
| `-l`  | `--list-only` |              |                          | Returns only the desired CPU type on stdout |
| `-v`  | `--verbose`   |              |                          | Enable verbose output |
| `-h`  | `--help`      |              |                          | Print help |

## Building from Source
Building ProxCLMC is easy and can be done by installing the following requirements for Rust/Cargo:
```bash
apt install -y cargo rustc libssh2-1-dev pkg-config
```

Afterwards, you can build the project from source:
```bash
cargo build --release
```
The resulting binary will be located at:
- `target/release/ProxCLMC`

## License
This project is licensed under the **GNU General Public License v3.0 or later**.
SPDX-License-Identifier: `GPL-3.0-or-later`

## Author
* Florian Paul Azim Hoberg [@gyptazy](https://github.com/gyptazy) (https://gyptazy.com)