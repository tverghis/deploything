"""Placeholder integration tests for agent commands.

These tests require a running agent to connect to the test server.
Run the agent before executing these tests:

    cd agent && cargo run

Then run the tests:

    uv run pytest tests/integration/
"""

import pytest

from agent_test_server.server import AgentTestServer, AgentConnection


@pytest.mark.skip(reason="Requires running agent - remove skip to run integration test")
async def test_run_command(agent_connection: AgentConnection) -> None:
    """Test sending a run command to the agent."""
    response = await agent_connection.send_run_command("nginx", "latest")
    assert response == "ok"


@pytest.mark.skip(reason="Requires running agent - remove skip to run integration test")
async def test_stop_command(agent_connection: AgentConnection) -> None:
    """Test sending a stop command to the agent."""
    response = await agent_connection.send_stop_command("test-container-id")
    # Response may be "ok" or an error if container doesn't exist
    assert isinstance(response, str)


async def test_server_starts_and_stops(test_server: AgentTestServer) -> None:
    """Test that the server can start and stop without errors."""
    assert test_server.host == "localhost"
    assert test_server.port == 4040
    assert test_server.connections == []
