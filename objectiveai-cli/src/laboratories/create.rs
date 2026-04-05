use bollard::exec::CreateExecOptions;
use bollard::models::ContainerCreateBody;
use bollard::query_parameters::{CreateContainerOptionsBuilder, UploadToContainerOptionsBuilder};

use super::create_args::CreateArgs;

pub async fn handle(args: CreateArgs) -> Result<crate::Output, crate::error::Error> {
    let docker = bollard::Docker::connect_with_local_defaults()?;
    let tar_bytes = mcp_tar(super::mcp_binary::MCP_BINARY);

    let futs: Vec<_> = args
        .builder_agent
        .iter()
        .enumerate()
        .map(|(i, _)| spawn_builder(&docker, &args.docker_image, i, &tar_bytes))
        .collect();

    let results = futures::future::join_all(futs).await;
    let mut container_ids = Vec::new();
    for result in results {
        container_ids.push(result?);
    }

    // MCP servers are now running in each container.
    // Next: communicate with them via the attached streams.
    unimplemented!()
}

/// Create a tar archive containing the MCP binary at the archive root.
fn mcp_tar(binary: &[u8]) -> Vec<u8> {
    let mut ar = tar::Builder::new(Vec::new());
    let mut header = tar::Header::new_gnu();
    header.set_size(binary.len() as u64);
    header.set_mode(0o755);
    header.set_cksum();
    ar.append_data(&mut header, "objectiveai-mcp", binary)
        .expect("failed to build tar archive");
    ar.into_inner().expect("failed to finalize tar archive")
}

/// Spawn a single builder container: create, start, upload MCP binary, and start the MCP server.
/// Returns the container ID.
async fn spawn_builder(
    docker: &bollard::Docker,
    image: &str,
    index: usize,
    mcp_tar: &[u8],
) -> Result<String, crate::error::Error> {
    // Create container with a keep-alive command
    let container_name = format!("objectiveai-lab-builder-{index}");
    let options = CreateContainerOptionsBuilder::default()
        .name(container_name.as_str())
        .build();

    let config = ContainerCreateBody {
        image: Some(image.to_string()),
        cmd: Some(vec!["sleep".to_string(), "infinity".to_string()]),
        ..Default::default()
    };

    let container = docker
        .create_container(Some(options), config)
        .await?;

    // Start the container
    docker.start_container(&container.id, None).await?;

    // Upload the MCP binary to the container root
    let upload_options = UploadToContainerOptionsBuilder::default()
        .path("/")
        .build();

    docker
        .upload_to_container(
            &container.id,
            Some(upload_options),
            bollard::body_full(mcp_tar.to_vec().into()),
        )
        .await?;

    // Start the MCP server inside the container
    let exec_options = CreateExecOptions {
        cmd: Some(vec!["/objectiveai-mcp"]),
        attach_stdin: Some(true),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        ..Default::default()
    };

    let exec = docker.create_exec(&container.id, exec_options).await?;
    let _start_result = docker.start_exec(&exec.id, None).await?;

    Ok(container.id)
}
