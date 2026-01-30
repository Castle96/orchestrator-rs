use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use models::ContainerConfig;

pub struct LxcConfig;

impl LxcConfig {
    pub fn lxc_root() -> PathBuf {
        std::env::var("LXC_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/var/lib/lxc"))
    }

    /// Generate LXC configuration file content
    pub fn generate(name: &str, config: &ContainerConfig) -> String {
        let lxc_root = Self::lxc_root();
        let mut lxc_config = String::new();

        // Basic container configuration
        lxc_config.push_str(&format!("lxc.uts.name = {}\n", name));
        lxc_config.push_str("lxc.arch = arm64\n");
        lxc_config.push_str("lxc.rootfs.path = dir:\n");
        lxc_config.push_str(&format!("lxc.rootfs.path = {}/{}/rootfs\n", lxc_root.display(), name));

        // CPU limits
        if let Some(cpu_limit) = config.cpu_limit {
            lxc_config.push_str(&format!("lxc.cgroup2.cpuset.cpus = 0-{}\n", cpu_limit - 1));
        }

        // Memory limits
        if let Some(memory_limit) = config.memory_limit {
            lxc_config.push_str(&format!("lxc.cgroup2.memory.max = {}\n", memory_limit));
        }

        // Network interfaces
        for (idx, net_if) in config.network_interfaces.iter().enumerate() {
            lxc_config.push_str(&format!("lxc.net.{}.type = veth\n", idx));
            lxc_config.push_str(&format!("lxc.net.{}.link = {}\n", idx, net_if.bridge));
            lxc_config.push_str(&format!("lxc.net.{}.name = {}\n", idx, net_if.name));
            if let Some(ref mac) = net_if.mac {
                lxc_config.push_str(&format!("lxc.net.{}.hwaddr = {}\n", idx, mac));
            }
        }

        // Environment variables
        for (key, value) in &config.environment {
            lxc_config.push_str(&format!("lxc.environment = {}={}\n", key, value));
        }

        lxc_config
    }

    /// Write configuration to file
    pub fn write(name: &str, config: &ContainerConfig) -> Result<()> {
        let config_dir = Self::lxc_root().join(name);
        fs::create_dir_all(&config_dir)
            .context("Failed to create container directory")?;

        let config_path = config_dir.join("config");
        let config_content = Self::generate(name, config);
        fs::write(&config_path, config_content)
            .context("Failed to write LXC config file")?;

        Ok(())
    }

    /// Read configuration from file
    pub fn read(name: &str) -> Result<String> {
        let config_path = Self::lxc_root().join(name).join("config");
        fs::read_to_string(&config_path)
            .context("Failed to read LXC config file")
    }
}
