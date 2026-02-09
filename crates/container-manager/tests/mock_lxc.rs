use std::fs::{self, File};
use std::io::Write;
use std::process::Command;

use models::{ContainerConfig, CreateContainerRequest};
use uuid::Uuid;
use container_manager::ContainerManager;

#[tokio::test]
async fn test_mock_container_create_and_list() {
    // Prepare a temporary directory for LXC root
    let base = std::env::temp_dir().join(format!("orchestrator_mock_{}", Uuid::new_v4()));
    let bin = base.join("bin");
    fs::create_dir_all(&bin).expect("create bin dir");

    // Path for the fake container state file
    let state_file = base.join("containers.txt");

    // Write helper scripts into bin
    let write_script = |name: &str, content: &str| {
        let p = bin.join(name);
        let mut f = File::create(&p).expect("create script");
        f.write_all(content.as_bytes()).expect("write");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perm = f.metadata().unwrap().permissions();
            perm.set_mode(0o755);
            fs::set_permissions(&p, perm).unwrap();
        }
    };

    // lxc-list: print the state file if exists
    write_script(
        "lxc-list",
        "#!/bin/sh\nif [ -f \"$LXC_STATE_FILE\" ]; then cat \"$LXC_STATE_FILE\"; fi\n",
    );

    // lxc-create: append the container name to the state file
    write_script(
        "lxc-create",
        "#!/bin/sh\nname=$1\necho $name >> \"$LXC_STATE_FILE\"\nexit 0\n",
    );

    // lxc-info: print a State line depending on presence in state file
    write_script(
        "lxc-info",
        "#!/bin/sh\nname=$1\nif [ -f \"$LXC_STATE_FILE\" ] && grep -q \"^$name$\" \"$LXC_STATE_FILE\"; then echo \"State: RUNNING\"; else echo \"State: STOPPED\"; fi\n",
    );

    // Prepend our bin to PATH and set LXC_ROOT and LXC_STATE_FILE
    let orig_path = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin.display(), orig_path);
    std::env::set_var("PATH", new_path);
    std::env::set_var("LXC_ROOT", base.display().to_string());
    std::env::set_var("LXC_STATE_FILE", state_file.display().to_string());

    // Debug prints to help diagnose permission issues in CI/local
    println!("LXC_ROOT={}", std::env::var("LXC_ROOT").unwrap());
    println!("LXC_STATE_FILE={}", std::env::var("LXC_STATE_FILE").unwrap());

    // Verify we can write to base
    let test_dir = base.join("probe_dir");
    fs::create_dir_all(&test_dir).expect("probe create_dir_all failed");
    let probe_file = test_dir.join("probe.txt");
    fs::write(&probe_file, b"probe").expect("probe write failed");

    // Quick exec check for our helper scripts
    let lxc_create_path = bin.join("lxc-create");
    println!("lxc-create path: {:?}", lxc_create_path);
    match Command::new(&lxc_create_path).arg("probe").output() {
        Ok(out) => {
            println!("script stdout: {}", String::from_utf8_lossy(&out.stdout));
            println!("script stderr: {}", String::from_utf8_lossy(&out.stderr));
        }
        Err(e) => {
            panic!("failed to exec script: {}", e);
        }
    }

    // Prepare a simple container config
    let config = ContainerConfig {
        cpu_limit: Some(1),
        memory_limit: Some(64 * 1024 * 1024),
        disk_limit: None,
        network_interfaces: vec![],
        rootfs_path: "".to_string(),
        environment: vec![],
    };

    let req = CreateContainerRequest {
        name: "test-container".to_string(),
        template: "busybox".to_string(),
        config: config.clone(),
    };

    // Call create
    let created = ContainerManager::create(req).await;
    assert!(created.is_ok(), "create failed: {:?}", created.err());

    // Ensure the state file contains the container
    let contents = fs::read_to_string(&state_file).expect("read state file");
    assert!(contents.contains("test-container"));

    // List via API and ensure the name appears
    let list = ContainerManager::list().await;
    assert!(list.is_ok());
    let names = list.unwrap();
    assert!(names.iter().any(|n| n == "test-container"));

    // Cleanup
    let _ = fs::remove_dir_all(&base);
}
