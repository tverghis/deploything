"""Command builder helpers for creating protobuf messages."""

from agent_test_server.proto.deploything.v1 import RemoteCommand, RunParams, StopParams


def build_run_command(image_name: str, tag: str | None = None) -> RemoteCommand:
    """Build a RemoteCommand with RunParams."""
    run_params = RunParams(image_name=image_name)
    if tag is not None:
        run_params.tag = tag
    return RemoteCommand(run=run_params)


def build_stop_command(container_id: str) -> RemoteCommand:
    """Build a RemoteCommand with StopParams."""
    stop_params = StopParams(container_id=container_id)
    return RemoteCommand(stop=stop_params)


def serialize_command(cmd: RemoteCommand) -> bytes:
    """Serialize a RemoteCommand to bytes for sending over WebSocket."""
    return cmd.SerializeToString()
