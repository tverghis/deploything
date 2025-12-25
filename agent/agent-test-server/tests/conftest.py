"""Pytest fixtures for agent integration tests."""

from collections.abc import AsyncGenerator

import pytest

from agent_test_server.server import AgentTestServer, AgentConnection


@pytest.fixture
async def test_server() -> AsyncGenerator[AgentTestServer, None]:
    """Fixture that starts and stops an AgentTestServer.

    Usage:
        async def test_something(test_server):
            # Server is already started on ws://localhost:4040
            # Start your agent, then use wait_for_connection
            pass
    """
    server = AgentTestServer(host="localhost", port=4040)
    await server.start()
    yield server
    await server.stop()


@pytest.fixture
async def agent_connection(test_server: AgentTestServer) -> AsyncGenerator[AgentConnection, None]:
    """Fixture that waits for an agent connection.

    NOTE: This fixture requires the agent to be running and connecting.
    It will timeout after 10 seconds if no agent connects.

    Usage:
        async def test_run_command(agent_connection):
            response = await agent_connection.send_run_command("nginx", "latest")
            assert response == "ok"
    """
    connection = await test_server.wait_for_connection(timeout=10.0)
    yield connection
