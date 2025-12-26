"""Format AgentSnapshot protobuf messages as human-readable text."""

from __future__ import annotations

from datetime import datetime, timezone

from agent_test_server.proto.deploything.v1 import (
    AgentSnapshot,
    ContainerState,
    ContainerStatus,
)


def _state_name(state: ContainerState) -> str:
    """Get human-readable state name."""
    mapping = {
        ContainerState.CONTAINER_STATE_UNSPECIFIED: "UNSPECIFIED",
        ContainerState.CONTAINER_STATE_RUNNING: "RUNNING",
        ContainerState.CONTAINER_STATE_EXITED: "EXITED",
    }
    return mapping.get(state, "UNKNOWN")


def _format_timestamp(snapshot: AgentSnapshot) -> str:
    """Format the snapshot timestamp as ISO string."""
    if snapshot.HasField("timestamp"):
        ts = snapshot.timestamp
        dt = datetime.fromtimestamp(ts.seconds + ts.nanos / 1e9, tz=timezone.utc)
        return dt.strftime("%Y-%m-%dT%H:%M:%SZ")
    return "(no timestamp)"


def _format_container(container: ContainerStatus, indent: str = "  ") -> str:
    """Format a single container status."""
    lines = []
    lines.append(f"{indent}- id: {container.id or '(none)'}")
    lines.append(f"{indent}  name: {container.name or '(none)'}")
    lines.append(f"{indent}  image: {container.image_ref or '(none)'}")
    lines.append(f"{indent}  state: {_state_name(container.container_state)}")
    return "\n".join(lines)


def format_snapshot(snapshot: AgentSnapshot) -> str:
    """Format an AgentSnapshot as human-readable multi-line text.

    Args:
        snapshot: The AgentSnapshot protobuf message.

    Returns:
        Multi-line string representation of the snapshot.
    """
    lines = ["AgentSnapshot:"]
    lines.append(f"  timestamp: {_format_timestamp(snapshot)}")

    if snapshot.container_status:
        lines.append("  containers:")
        for container in snapshot.container_status:
            lines.append(_format_container(container, indent="    "))
    else:
        lines.append("  containers: (none)")

    return "\n".join(lines)
