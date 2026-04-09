use anyhow::Result;

/// A Docker container with its exposed host ports.
#[derive(Debug, Clone)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub host_ports: Vec<u16>,
    pub status: String,
}

/// Collect running Docker containers and their port mappings.
/// Returns an empty vec (not an error) if Docker is not available.
pub async fn collect() -> Result<Vec<ContainerInfo>> {
    // Check docker daemon is reachable before connecting
    if which::which("docker").is_err() {
        tracing::debug!("docker not found in PATH, skipping container collection");
        return Ok(Vec::new());
    }

    match collect_from_daemon().await {
        Ok(containers) => Ok(containers),
        Err(e) => {
            tracing::debug!("docker daemon unavailable: {e}");
            Ok(Vec::new())
        }
    }
}

async fn collect_from_daemon() -> Result<Vec<ContainerInfo>> {
    use bollard::Docker;
    use bollard::container::ListContainersOptions;
    use std::collections::HashMap;

    let docker = Docker::connect_with_local_defaults()?;

    let options = ListContainersOptions::<String> {
        all: false,
        filters: HashMap::new(),
        ..Default::default()
    };

    let containers = docker.list_containers(Some(options)).await?;

    let infos = containers
        .into_iter()
        .map(|c| {
            let name = c
                .names
                .as_deref()
                .and_then(|n| n.first())
                .map(|n| n.trim_start_matches('/').to_string())
                .unwrap_or_default();

            let host_ports = c
                .ports
                .as_deref()
                .unwrap_or_default()
                .iter()
                .filter_map(|p| p.public_port)
                .collect();

            ContainerInfo {
                id: c.id.unwrap_or_default(),
                name,
                image: c.image.unwrap_or_default(),
                host_ports,
                status: c.status.unwrap_or_default(),
            }
        })
        .collect();

    Ok(infos)
}
