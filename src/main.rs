use anyhow::{Context, Result};
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;
use std::process::Command;

#[tokio::main]
async fn main() -> Result<()> {
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = Client::new(&config);

    let app_name = env::var("APP_NAME").context("APP_NAME environment variable not set")?;
    let bucket = "prod-dotzero-project-builds";
    let latest_key = format!("{}/latest.txt", app_name);
    let shared_dir = "/shared";
    let cmd_path = format!("{}/cmd", shared_dir);
    let new_cmd_path = format!("{}/new_cmd", shared_dir);
    let config_path = format!("{}/config.json", shared_dir);
    let new_config_path = format!("{}/new_config.json", shared_dir);
    let pid_path = format!("{}/wrapper.pid", shared_dir);
    let current_version_path = format!("{}/current_version.txt", shared_dir);

    // Initial download if binary missing
    if !Path::new(&cmd_path).exists() {
        download_latest(&client, bucket, &app_name, &latest_key, &current_version_path, &cmd_path, &config_path).await?;
    }

    // Poll loop
    loop {
        let latest_version = get_object(&client, bucket, &latest_key).await?.trim().to_string();

        let current_version = fs::read_to_string(&current_version_path)
            .unwrap_or_default()
            .trim()
            .to_string();

        if latest_version != current_version {
            // Download binary to temp path
            let binary_key = format!("{}/executables/{}/cmd", app_name, latest_version);
            let binary_data = get_object_bytes(&client, bucket, &binary_key).await?;
            let mut file = File::create(&new_cmd_path)?;
            file.write_all(&binary_data)?;
            fs::set_permissions(&new_cmd_path, fs::Permissions::from_mode(0o755))?;

            // Download config to temp path
            let config_key = format!("{}/executables/{}/config.json", app_name, latest_version);
            let config_data = get_object_bytes(&client, bucket, &config_key).await?;
            let mut file = File::create(&new_config_path)?;
            file.write_all(&config_data)?;

            // Atomic moves
            fs::rename(&new_cmd_path, &cmd_path)?;
            fs::rename(&new_config_path, &config_path)?;

            // Update current version
            fs::write(&current_version_path, &latest_version)?;

            // Send signal
            if Path::new(&pid_path).exists() {
                let pid_str = fs::read_to_string(&pid_path)?.trim().to_string();
                let pid: i32 = pid_str.parse()?;
                Command::new("kill")
                    .arg("-USR1")
                    .arg(pid.to_string())
                    .output()
                    .context("Failed to send SIGUSR1")?;
            }
        }

        sleep(Duration::from_secs(3)).await;
    }
}

async fn download_latest(client: &Client, bucket: &str, app_name: &str, latest_key: &str, version_path: &str, cmd_path: &str, config_path: &str) -> Result<()> {
    let latest_version = get_object(client, bucket, latest_key).await?.trim().to_string();
    let binary_key = format!("{}/executables/{}/cmd", app_name, latest_version);
    let binary_data = get_object_bytes(client, bucket, &binary_key).await?;
    let mut file = File::create(cmd_path)?;
    file.write_all(&binary_data)?;
    fs::set_permissions(cmd_path, fs::Permissions::from_mode(0o755))?;

    let config_key = format!("{}/executables/{}/config.json", app_name, latest_version);
    let config_data = get_object_bytes(client, bucket, &config_key).await?;
    let mut file = File::create(config_path)?;
    file.write_all(&config_data)?;

    fs::write(version_path, &latest_version)?;
    Ok(())
}

async fn get_object(client: &Client, bucket: &str, key: &str) -> Result<String> {
    let resp = client.get_object().bucket(bucket).key(key).send().await?;
    let data = resp.body.collect().await?;
    Ok(String::from_utf8(data.into_bytes().to_vec())?)
}

async fn get_object_bytes(client: &Client, bucket: &str, key: &str) -> Result<Vec<u8>> {
    let resp = client.get_object().bucket(bucket).key(key).send().await?;
    let data = resp.body.collect().await?;
    Ok(data.into_bytes().to_vec())
}