# ProxCLMC
<img align="center" src="https://cdn.gyptazy.com/img/ProxCLMC_logo.png"/>
<br>
<p float="center"><img src="https://img.shields.io/github/license/gyptazy/ProxCLMC"/><img src="https://img.shields.io/github/contributors/gyptazy/ProxCLMC"/><img src="https://img.shields.io/github/last-commit/gyptazy/ProxCLMC/main"/><img src="https://img.shields.io/github/issues-raw/gyptazy/ProxCLMC"/><img src="https://img.shields.io/github/issues-pr/gyptazy/ProxCLMC"/></p>


## General
**ProxCLMC** is a lightweight tool to determine the **maximum CPU compatibility level** that is supported across all nodes in a **Proxmox VE cluster**.

It remotely inspects each cluster node via SSH, analyzes CPU capabilities, and calculates the **lowest common x86-64 CPU baseline** to ensure safe live migration and predictable VM behavior.

## Why ProxCLMC?
In mixed-hardware Proxmox clusters, CPU feature mismatches are a common source of:

- Failed live migrations
- Unstable VM behavior
- Severe performance degradation (especially for Windows VMs when using the `host` CPU type)

ProxCLMC provides a **deterministic, cluster-wide answer** to the question:

> Which CPU type can I safely use for all VMs in this cluster?

## Features
- Parses cluster nodes directly from `corosync.conf`
- Connects to each node via SSH
- Reads `/proc/cpuinfo` remotely
- Extracts CPU flags
- Maps flags to x86-64 CPU baselines
- Calculates the **lowest common CPU type** across all nodes
- Fully compatible with Proxmox / QEMU CPU models:
  - `x86-64-v1`
  - `x86-64-v2`
  - `x86-64-v3`
  - `x86-64-v4`

## How it works
1. Parse the `nodelist` from `corosync.conf`
2. Connect to each node via SSH using key-based authentication
3. Read `/proc/cpuinfo`
4. Extract CPU flags
5. Derive the supported x86-64 CPU baseline per node
6. Select the **lowest common denominator** for the cluster

This mirrors the CPU compatibility validation to Proxmox VE based clusters, which might already be known from other virtualiaztion solutions. 

## Example Output
All available nodes in a Proxmox Cluster will be listed by their supported CPU model. Afterwards, the maximum usable CPU type for the cluster will be printed that can be used for VMs to ensure safe live-migrations:
```
test-pmx01 | 10.10.10.21 | x86-64-v3
test-pmx02 | 10.10.10.22 | x86-64-v3

Cluster CPU type: x86-64-v3
```

## Requirements
- Proxmox VE cluster
- SSH key-based access to all cluster nodes

## Install & Usage

```
wget https://cdn.gyptazy.com/debian/proxclmc/proxclmc_1.0.0_amd64.deb ; dpkg -i proxclmc_1.0.0_amd64.deb ; proxclmc
```

## Options
The following cli options are available.

| Short | Long        | Argument     | Default                 | Description        |
|------:|-------------|--------------|--------------------------|--------------------|
| `-s`  | `--ssh-file`| `<SSH_FILE>` | `/root/.ssh/id_rsa`      | SSH private key to use |
|       | `--version` |              |                          | Print version information |
| `-v`  | `--verbose` |              |                          | Enable verbose output |
| `-h`  | `--help`    |              |                          | Print help |


## Build
Building ProxCLMC is easy and can be done by installing the following requirements for Rust/Cargo:
```
apt install -y cargo rustc libssh2-1-dev pkg-config
```

Afterwards, you can build the project from source:
```
cargo build --release
```
The resulting binary will be located at:
- `target/release/ProxCLMC`

## Supported CPU baselines
| CPU Type   | Description                           |
|-----------|---------------------------------------|
| x86-64-v1 | Legacy baseline (SSE2)                |
| x86-64-v2 | Modern baseline (SSE4.2, POPCNT)      |
| x86-64-v3 | AVX / AVX2 capable                    |
| x86-64-v4 | AVX-512 capable (usually not advised) |

## Recommended usage
For most mixed-hardware Proxmox clusters:
- Prefer **x86-64-v2** as a safe default
- Avoid the `host` CPU type for Windows VMs
- Avoid cluster-wide AVX-512 unless explicitly required

## License
This project is licensed under the **GNU General Public License v3.0 or later**.
SPDX-License-Identifier: `GPL-3.0-or-later`

## Author
* Florian Paul Azim Hoberg @gyptazy (https://gyptazy.com)