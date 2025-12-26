"""Agent connection handler for sending commands and receiving snapshots."""

from __future__ import annotations

from typing import TYPE_CHECKING, Awaitable, Callable

if TYPE_CHECKING:
    from websockets.asyncio.server import ServerConnection

from agent_test_server.commands.builders import (
    build_run_command,
    build_stop_command,
    serialize_command,
)
from agent_test_server.proto.deploything.v1 import AgentSnapshot


# Type alias for snapshot callback
SnapshotCallback = Callable[[AgentSnapshot], Awaitable[None]]


class AgentConnection:
    """Represents a connection to a deploything agent."""

    def __init__(
        self,
        websocket: ServerConnection,
        snapshot_callback: SnapshotCallback | None = None,
    ) -> None:
        self._websocket = websocket
        self._snapshot_callback = snapshot_callback
        self._closed = False

    @property
    def is_closed(self) -> bool:
        """Check if the connection is closed."""
        return self._closed

    def set_snapshot_callback(self, callback: SnapshotCallback | None) -> None:
        """Set the callback for receiving snapshots."""
        self._snapshot_callback = callback

    async def send_run_command(
        self,
        image_name: str,
        tag: str | None = None,
        port_mapping: str | None = None,
    ) -> None:
        """Send a run command to the agent.

        Args:
            image_name: Docker image name to run.
            tag: Optional image tag (e.g., "latest").
            port_mapping: Optional port mapping in format 'container_port[/protocol]:host_port'
                          e.g., '8080/tcp:8080'
        """
        cmd = build_run_command(image_name, tag, port_mapping)
        data = serialize_command(cmd)
        await self._websocket.send(data)

    async def send_stop_command(self, container_id: str) -> None:
        """Send a stop command to the agent.

        Args:
            container_id: ID of the container to stop.
        """
        cmd = build_stop_command(container_id)
        data = serialize_command(cmd)
        await self._websocket.send(data)

    async def _run(self) -> None:
        """Internal method to receive messages and invoke snapshot callback."""
        try:
            async for message in self._websocket:
                if isinstance(message, bytes):
                    snapshot = AgentSnapshot()
                    snapshot.ParseFromString(message)
                    if self._snapshot_callback is not None:
                        await self._snapshot_callback(snapshot)
        finally:
            self._closed = True
