"""Interactive TUI for testing the deploything agent."""

from __future__ import annotations

import asyncio

import click

from agent_test_server.cli.tui import TUIApplication
from agent_test_server.server import AgentTestServer


async def run_cli(host: str, port: int) -> None:
    """Run the interactive TUI."""
    server = AgentTestServer(host=host, port=port)

    # Create TUI
    tui = TUIApplication(server)

    # Set snapshot callback on server
    server.set_snapshot_callback(tui.handle_snapshot)

    await server.start()

    # Monitor connection status in background
    async def monitor_connection() -> None:
        while True:
            connections = server.connections
            if any(not c.is_closed for c in connections):
                tui.set_connection_status("connected")
            else:
                tui.set_connection_status("disconnected")
            await asyncio.sleep(1)

    monitor_task = asyncio.create_task(monitor_connection())

    try:
        await tui.run()
    finally:
        monitor_task.cancel()
        try:
            await monitor_task
        except asyncio.CancelledError:
            pass
        await server.stop()


@click.command()
@click.option("--host", default="localhost", help="Host to bind to")
@click.option("--port", default=4040, help="Port to listen on")
def main(host: str, port: int) -> None:
    """Interactive TUI for testing the deploything agent."""
    try:
        asyncio.run(run_cli(host, port))
    except KeyboardInterrupt:
        pass
    print("\nGoodbye!")


if __name__ == "__main__":
    main()
