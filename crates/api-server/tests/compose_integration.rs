use std::env;
use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};

#[tokio::test]
async fn compose_dev_smoke_tests() -> Result<()> {
    // This test is intentionally opt-in. Set RUN_COMPOSE_TESTS=1 to allow the test
    // to bring up the docker-compose dev stack. Otherwise the test will attempt to
    // hit localhost:8080 which must already be running (e.g. developer started it).
    let run_compose = env::var("RUN_COMPOSE_TESTS").unwrap_or_default() == "1";

    let repo_root = env::var("ORCHESTRATOR_RS_ROOT").unwrap_or_else(|_| {
        // default to project root relative to crate (two levels up)
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let p = manifest
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| manifest.to_path_buf())
            .canonicalize()
            .expect("failed to canonicalize repo root");
        p.to_string_lossy().into_owned()
    });

    if run_compose {
        // Bring up the dev compose stack using the repo compose files.
        let status = Command::new("docker")
            .args([
                "compose",
                "-f",
                "docker-compose.yml",
                "-f",
                "docker-compose.dev.yml",
                "up",
                "--build",
                "-d",
            ])
            .current_dir(&repo_root)
            .status()
            .context("failed to run docker compose up")?;
        if !status.success() {
            bail!("docker compose up failed");
        }
    }

    // Ensure we tear down compose if we started it.
    let teardown = run_compose;

    let res = async {
        // Wait for API to become healthy (up to 120s)
        let client = reqwest::Client::new();
        let deadline = Instant::now() + Duration::from_secs(120);
        let mut attempts = 0;

        println!("Waiting for API to become healthy...");

        loop {
            attempts += 1;
            if Instant::now() > deadline {
                bail!(
                    "timeout waiting for API health endpoint after {} attempts",
                    attempts
                );
            }

            println!("Attempt {}: Checking health endpoint...", attempts);

            match client.get("http://localhost:8080/health").send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let text = resp.text().await.unwrap_or_default();
                        println!("Health response: {}", text);
                        if text.contains("healthy") || text.contains("skipped system checks") {
                            println!("API is healthy! ✅");
                            break;
                        } else {
                            println!("API responded but not healthy yet: {}", text);
                        }
                    } else {
                        println!("Health check failed with status: {}", resp.status());
                    }
                }
                Err(e) => {
                    println!("Health check request failed: {}", e);
                }
            }

            if attempts < 60 {
                // Max 2 minutes with 2-second intervals
                tokio::time::sleep(Duration::from_secs(2)).await;
            } else {
                bail!(
                    "timeout waiting for API health endpoint after {} attempts",
                    attempts
                );
            }
        }

        // Basic smoke requests
        println!("Running smoke tests...");

        match client.get("http://localhost:8080/health").send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    println!("✅ /health endpoint: {}", resp.status());
                } else {
                    bail!("/health returned non-success: {}", resp.status());
                }
            }
            Err(e) => {
                bail!("Health check failed: {}", e);
            }
        }

        match client.get("http://localhost:8080/ready").send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    println!("✅ /ready endpoint: {}", resp.status());
                } else {
                    bail!("/ready returned non-success: {}", resp.status());
                }
            }
            Err(e) => {
                bail!("Ready check failed: {}", e);
            }
        }

        println!("✅ All smoke tests passed!");

        let resp = client.get("http://localhost:8080/ready").send().await?;
        if !resp.status().is_success() {
            bail!("/ready returned non-success");
        }

        Ok(()) as Result<()>
    }
    .await;

    if teardown {
        let _ = Command::new("docker")
            .args([
                "compose",
                "-f",
                "docker-compose.yml",
                "-f",
                "docker-compose.dev.yml",
                "down",
            ])
            .current_dir(&repo_root)
            .status();
    }

    res
}
