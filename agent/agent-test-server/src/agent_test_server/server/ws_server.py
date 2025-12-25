"""WebSocket server for accepting agent connections."""

from __future__ import annotations

import asyncio

from websockets.asyncio.server import serve, Server, ServerConnection

from agent_test_server.server.connection import AgentConnection


class AgentTestServer:
    """WebSocket server that accepts connections from the deploything agent."""

    def __init__(self, host: str = "localhost", port: int = 4040) -> None:
        self._host = host
        self._port = port
        self._server: Server | None = None
        self._connections: list[AgentConnection] = []
        self._connection_event = asyncio.Event()

    @property
    def host(self) -> str:
        return self._host

    @property
    def port(self) -> int:
        return self._port

    @property
    def connections(self) -> list[AgentConnection]:
        """List of active agent connections."""
        return list(self._connections)

    async def start(self) -> None:
        """Start the WebSocket server."""
        self._server = await serve(
            self._handle_connection,
            self._host,
            self._port,
        )

    async def stop(self) -> None:
        """Stop the server and close all connections."""
        if self._server is not None:
            self._server.close()
            await self._server.wait_closed()
            self._server = None
        self._connections.clear()

    async def wait_for_connection(self, timeout: float | None = None) -> AgentConnection:
        """Wait for an agent to connect.

        Args:
            timeout: Maximum time to wait in seconds. None means wait forever.

        Returns:
            The AgentConnection for the connected agent.

        Raises:
            asyncio.TimeoutError: If timeout expires before a connection is made.
        """
        if self._connections:
            return self._connections[0]

        self._connection_event.clear()
        await asyncio.wait_for(self._connection_event.wait(), timeout=timeout)
        return self._connections[-1]

    async def _handle_connection(self, websocket: ServerConnection) -> None:
        """Handle a new WebSocket connection."""
        connection = AgentConnection(websocket)
        self._connections.append(connection)
        self._connection_event.set()

        try:
            await connection._run()
        finally:
            if connection in self._connections:
                self._connections.remove(connection)
