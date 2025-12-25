"""Interactive CLI for testing the deploything agent."""

from __future__ import annotations

import asyncio
import time

import click
from prompt_toolkit import PromptSession
from prompt_toolkit.auto_suggest import AutoSuggestFromHistory
from prompt_toolkit.history import InMemoryHistory

from agent_test_server.server import AgentTestServer, AgentConnection


class InteractiveCLI:
    """Interactive command-line interface for sending commands to the agent."""

    DOUBLE_CTRL_C_TIMEOUT = 1.0  # seconds

    def __init__(self, server: AgentTestServer) -> None:
        self._server = server
        self._containers: list[str] = []
        self._last_interrupt_time: float = 0.0
        self._session: PromptSession[str] = PromptSession(
            history=InMemoryHistory(),
            auto_suggest=AutoSuggestFromHistory(),
        )

    def _get_connection(self) -> AgentConnection | None:
        """Get the current active connection, if any."""
        connections = self._server.connections
        for conn in connections:
            if not conn.is_closed:
                return conn
        return None

    def _get_prompt(self) -> str:
        """Get the prompt string based on connection status."""
        if self._get_connection() is None:
            return "[disconnected] > "
        return "[connected] > "

    async def run(self) -> None:
        """Run the interactive CLI loop."""
        print(f"Agent Test Server listening on ws://{self._server.host}:{self._server.port}")
        print("Waiting for agent to connect...")

        try:
            await self._server.wait_for_connection(timeout=None)
            print("Agent connected!")
        except asyncio.CancelledError:
            return

        print("Type 'help' for available commands.\n")

        while True:
            try:
                line = await self._session.prompt_async(self._get_prompt())
                line = line.strip()

                if not line:
                    continue

                await self._handle_command(line)

            except KeyboardInterrupt:
                now = time.monotonic()
                if now - self._last_interrupt_time < self.DOUBLE_CTRL_C_TIMEOUT:
                    print("\nInterrupted.")
                    break
                self._last_interrupt_time = now
                print("\nPress Ctrl+C again to quit.")
                continue
            except EOFError:
                break

    async def _handle_command(self, line: str) -> None:
        """Parse and handle a command."""
        parts = line.split()
        cmd = parts[0].lower()
        args = parts[1:]

        if cmd == "help":
            self._print_help()
        elif cmd == "run":
            await self._cmd_run(args)
        elif cmd == "stop":
            await self._cmd_stop(args)
        elif cmd == "status":
            self._cmd_status()
        elif cmd == "list":
            self._cmd_list()
        elif cmd in ("quit", "exit"):
            raise EOFError()
        else:
            print(f"Unknown command: {cmd}. Type 'help' for available commands.")

    def _print_help(self) -> None:
        """Print help message."""
        print("""
Available commands:
  help              Show this help message
  run <image> [tag] Send run command (e.g., run nginx latest)
  stop <id>         Send stop command for a container
  status            Show connection status
  list              List containers started this session
  quit              Exit the CLI
""")

    async def _cmd_run(self, args: list[str]) -> None:
        """Handle the run command."""
        if not args:
            print("Usage: run <image> [tag]")
            return

        connection = self._get_connection()
        if connection is None:
            print("Error: No agent connected")
            return

        image = args[0]
        tag = args[1] if len(args) > 1 else None

        print(f"Sending run command: image={image}, tag={tag or '(none)'}")
        try:
            response = await connection.send_run_command(image, tag)
            print(f"Response: {response}")
            if response == "ok":
                self._containers.append(f"{image}:{tag or 'latest'}")
        except Exception as e:
            print(f"Error: {e}")

    async def _cmd_stop(self, args: list[str]) -> None:
        """Handle the stop command."""
        if not args:
            print("Usage: stop <container_id>")
            return

        connection = self._get_connection()
        if connection is None:
            print("Error: No agent connected")
            return

        container_id = args[0]
        print(f"Sending stop command: container_id={container_id}")
        try:
            response = await connection.send_stop_command(container_id)
            print(f"Response: {response}")
        except Exception as e:
            print(f"Error: {e}")

    def _cmd_status(self) -> None:
        """Show connection status."""
        if self._get_connection() is None:
            print("Status: Disconnected")
        else:
            print("Status: Connected")

    def _cmd_list(self) -> None:
        """List containers started this session."""
        if not self._containers:
            print("No containers started this session.")
            return
        print("Containers started this session:")
        for i, container in enumerate(self._containers, 1):
            print(f"  {i}. {container}")


async def run_cli(host: str, port: int) -> None:
    """Run the interactive CLI."""
    server = AgentTestServer(host=host, port=port)
    await server.start()

    cli = InteractiveCLI(server)
    try:
        await cli.run()
    finally:
        await server.stop()


@click.command()
@click.option("--host", default="localhost", help="Host to bind to")
@click.option("--port", default=4040, help="Port to listen on")
def main(host: str, port: int) -> None:
    """Interactive CLI for testing the deploything agent."""
    try:
        asyncio.run(run_cli(host, port))
    except KeyboardInterrupt:
        pass
    print("\nGoodbye!")


if __name__ == "__main__":
    main()
