# LXC Operations Modification Guide

This document explains how to modify LXC operations in the orchestrator to handle permission issues.

## Problem
The orchestrator requires root privileges to execute LXC commands, which causes permission errors when running as a regular user.

## Solutions

### Option 1: Enhanced LXC Command Implementation ✅ RECOMMENDED

**File**: `crates/container-manager/src/lxc.rs`

**Features**:
- Detects if running as root
- Tries direct execution first
- Falls back to passwordless sudo
- Provides clear error messages
- No password required when properly configured

**Implementation**:
```rust
impl LxcCommand {
    fn is_root() -> bool {
        nix::unistd::getuid().is_root()
    }

    pub fn execute(args: &[&str]) -> Result<String> {
        if Self::is_root() {
            return Self::execute_direct(&cmd_name, &args[1..]);
        }
        
        match Self::execute_with_sudo(&cmd_name, &args[1..]) {
            Ok(output) => Ok(output),
            Err(e) => Err(anyhow::anyhow!(
                "LXC operations require root privileges. {}",
                e
            ))
        }
    }
}
```

### Option 2: Passwordless Sudo Configuration

**Script**: `setup_sudo.sh`

**Steps**:
1. Run `./setup_sudo.sh`
2. This creates `/etc/sudoers.d/lxc-<username>`
3. Allows passwordless execution of LXC commands

**Manual setup**:
```bash
echo "$(whoami) ALL=(ALL) NOPASSWD: /usr/bin/lxc-*" | sudo tee /etc/sudoers.d/lxc-user
```

### Option 3: Run as Root User

Simplest solution - run the entire orchestrator as root:
```bash
sudo cargo run --bin api-server
```

### Option 4: User Namespaces (Rootless Containers)

More complex but allows unprivileged containers:
```bash
# Modify container creation to use user namespaces
lxc-create -t download -n container1 -- --mapped-uid 1000 --mapped-gid 1000
```

### Option 5: Docker Integration

Replace LXC with Docker for better permission handling:
- Docker has socket-based access
- Better user namespace support
- More mature ecosystem

## Implementation Steps

### 1. Update Dependencies
Add to `crates/container-manager/Cargo.toml`:
```toml
nix = { workspace = true }
```

### 2. Replace LXC Implementation
Replace `crates/container-manager/src/lxc.rs` with the enhanced version.

### 3. Build and Test
```bash
cd crates/container-manager && cargo build
cd crates/api-server && cargo build
```

### 4. Configure Sudo (Option 2)
```bash
./setup_sudo.sh
```

### 5. Restart Orchestrator
```bash
cd crates/api-server && cargo run
```

## Usage Examples

### With Enhanced Implementation
```bash
# Running as regular user (with sudo configured)
./container_deployment_demo.sh

# Running as root
sudo ./container_deployment_demo.sh

# Without any setup (will show clear error message)
./container_deployment_demo.sh
# Output: "LXC operations require root privileges..."
```

### API Testing
```bash
# Check health
curl http://localhost:8080/health

# Create container
curl -X POST http://localhost:8080/api/v1/containers \
  -H 'Content-Type: application/json' \
  -d '{"name": "test", "template": "ubuntu", "config": {...}}'
```

## Security Considerations

### Passwordless Sudo
- Only allows LXC commands
- Specific to your user
- Can be easily removed
- Follows principle of least privilege

### Root Operation
- Full system access
- Higher security risk
- Easier to implement
- Good for development/testing

### User Namespaces
- Most secure
- Complex setup
- Limited container functionality
- Best for production

## Troubleshooting

### Common Issues
1. **"sudo: a password is required"**
   - Run `./setup_sudo.sh` or configure manually

2. **"Permission denied"**
   - Check if running as root or sudo configured

3. **"LXC command failed"**
   - Verify LXC is installed
   - Check kernel support with `lxc-checkconfig`

### Debug Commands
```bash
# Check user
whoami

# Check sudo access
sudo -n /usr/bin/lxc-ls

# Check LXC support
lxc-checkconfig

# Check running processes
ps aux | grep api-server
```

## Files Modified

1. `crates/container-manager/src/lxc.rs` - Enhanced LXC command execution
2. `crates/container-manager/Cargo.toml` - Added nix dependency
3. `setup_sudo.sh` - Passwordless sudo configuration
4. `lxc_modification_guide.sh` - Comprehensive setup guide

## Recommendation

**Use Option 1 (Enhanced Implementation) + Option 2 (Passwordless Sudo)** for the best balance of:
- ✅ Security (limited sudo access)
- ✅ Usability (works without root)
- ✅ Clear error messages
- ✅ Backward compatibility (still works as root)

This gives you a production-ready solution that's secure and easy to use!