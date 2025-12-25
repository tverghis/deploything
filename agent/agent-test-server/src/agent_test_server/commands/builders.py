"""Command builder helpers for creating protobuf messages."""

from agent_test_server.proto.deploything.v1 import (
    ContainerHostConfig,
    PortMap,
    RemoteCommand,
    RunParams,
    StopParams,
)


def parse_port_mapping(port_mapping: str) -> PortMap:
    """Parse a port mapping string like '8080/tcp:8080' into a PortMap.

    Args:
        port_mapping: String in format 'container_port[/protocol]:host_port'
                      e.g., '8080/tcp:8080' or '3000:3000'

    Returns:
        A PortMap with from and to fields set.
    """
    parts = port_mapping.split(":")
    if len(parts) != 2:
        raise ValueError(f"Invalid port mapping format: {port_mapping}")
    return PortMap(**{"from": parts[0], "to": parts[1]})


def build_run_command(
    image_name: str,
    tag: str | None = None,
    port_mapping: str | None = None,
) -> RemoteCommand:
    """Build a RemoteCommand with RunParams.

    Args:
        image_name: Docker image name to run.
        tag: Optional image tag (e.g., "latest").
        port_mapping: Optional port mapping in format 'container_port[/protocol]:host_port'
                      e.g., '8080/tcp:8080'
    """
    run_params = RunParams(image_name=image_name)
    if tag is not None:
        run_params.tag = tag
    if port_mapping is not None:
        port_map = parse_port_mapping(port_mapping)
        run_params.container_host_config.CopyFrom(ContainerHostConfig(port_map=port_map))
    return RemoteCommand(run=run_params)


def build_stop_command(container_id: str) -> RemoteCommand:
    """Build a RemoteCommand with StopParams."""
    stop_params = StopParams(container_id=container_id)
    return RemoteCommand(stop=stop_params)


def serialize_command(cmd: RemoteCommand) -> bytes:
    """Serialize a RemoteCommand to bytes for sending over WebSocket."""
    return cmd.SerializeToString()
