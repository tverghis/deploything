"""Agent connection handler for sending commands and receiving responses."""

from __future__ import annotations

import asyncio
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from websockets.asyncio.server import ServerConnection

from agent_test_server.commands.builders import (
    build_run_command,
    build_stop_command,
    serialize_command,
)
from agent_test_server.proto.deploything.v1 import RemoteCommand


class AgentConnection:
    """Represents a connection to a deploything agent."""

    def __init__(self, websocket: ServerConnection) -> None:
        self._websocket = websocket
        self._response_queue: asyncio.Queue[str] = asyncio.Queue()
        self._closed = False

    @property
    def is_closed(self) -> bool:
        """Check if the connection is closed."""
        return self._closed

    async def send_run_command(self, image_name: str, tag: str | None = None) -> str:
        """Send a run command to the agent.

        Args:
            image_name: Docker image name to run.
            tag: Optional image tag (e.g., "latest").

        Returns:
            Response from the agent ("ok" or error message).
        """
        cmd = build_run_command(image_name, tag)
        return await self._send_and_receive(cmd)

    async def send_stop_command(self, container_id: str) -> str:
        """Send a stop command to the agent.

        Args:
            container_id: ID of the container to stop.

        Returns:
            Response from the agent ("ok" or error message).
        """
        cmd = build_stop_command(container_id)
        return await self._send_and_receive(cmd)

    async def _send_and_receive(self, cmd: RemoteCommand) -> str:
        """Serialize and send a command, then wait for response.

        Args:
            cmd: The RemoteCommand to send.

        Returns:
            Text response from the agent.
        """
        data = serialize_command(cmd)
        await self._websocket.send(data)
        response = await self._response_queue.get()
        return response

    async def _run(self) -> None:
        """Internal method to receive messages and queue responses."""
        try:
            async for message in self._websocket:
                if isinstance(message, str):
                    await self._response_queue.put(message)
        finally:
            self._closed = True
