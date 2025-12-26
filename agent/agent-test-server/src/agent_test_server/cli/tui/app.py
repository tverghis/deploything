"""Split-screen TUI application using prompt_toolkit."""

from __future__ import annotations

from datetime import datetime
from typing import TYPE_CHECKING

from prompt_toolkit import Application
from prompt_toolkit.buffer import Buffer
from prompt_toolkit.document import Document
from prompt_toolkit.key_binding import KeyBindings
from prompt_toolkit.layout import (
    BufferControl,
    FormattedTextControl,
    HSplit,
    Layout,
    Window,
)
from prompt_toolkit.widgets import Frame, TextArea

from agent_test_server.proto.deploything.v1 import AgentSnapshot
from agent_test_server.snapshot import format_snapshot

if TYPE_CHECKING:
    from agent_test_server.server import AgentTestServer


class TUIApplication:
    """Split-screen TUI with activity log and command input."""

    def __init__(self, server: AgentTestServer) -> None:
        self._server = server
        self._log_lines: list[str] = []
        self._connection_status = "disconnected"

        # Create UI components
        self._log_buffer = Buffer(read_only=True)
        self._input_area = TextArea(
            height=1,
            prompt=">>> ",
            multiline=False,
            wrap_lines=False,
        )

        # Key bindings
        self._kb = KeyBindings()
        self._setup_keybindings()

        # Build layout
        self._layout = self._build_layout()

        # Create application
        self._app: Application[None] = Application(
            layout=self._layout,
            key_bindings=self._kb,
            full_screen=True,
            mouse_support=True,
        )

    def _setup_keybindings(self) -> None:
        """Setup key bindings."""

        @self._kb.add("c-c")
        def _exit_ctrl_c(event) -> None:
            """Exit on Ctrl-C."""
            event.app.exit()

        @self._kb.add("c-d")
        def _exit_ctrl_d(event) -> None:
            """Exit on Ctrl-D."""
            event.app.exit()

        @self._kb.add("enter")
        async def _handle_enter(event) -> None:
            """Handle command input."""
            text = self._input_area.text.strip()
            self._input_area.text = ""
            if text:
                await self._handle_command(text)

    def _build_layout(self) -> Layout:
        """Build the TUI layout."""
        # Status bar at top
        status_bar = Window(
            content=FormattedTextControl(self._get_status_text),
            height=1,
            style="reverse",
        )

        # Activity log (scrollable)
        log_window = Frame(
            Window(
                content=BufferControl(buffer=self._log_buffer),
                wrap_lines=True,
            ),
            title="Activity Log",
        )

        # Input area at bottom
        input_frame = Frame(
            self._input_area,
            title="Command",
            height=3,
        )

        # Combine into vertical split
        root = HSplit([
            status_bar,
            log_window,  # Takes remaining space
            input_frame,
        ])

        return Layout(root, focused_element=self._input_area)

    def _get_status_text(self) -> str:
        """Get the status bar text."""
        host = self._server.host
        port = self._server.port
        status = self._connection_status
        return f" deploything agent-test-server | ws://{host}:{port} | Status: {status} "

    def _append_log(self, message: str, prefix: str = "") -> None:
        """Append a message to the activity log."""
        timestamp = datetime.now().strftime("%H:%M:%S")
        if prefix:
            line = f"[{timestamp}] {prefix} {message}"
        else:
            line = f"[{timestamp}] {message}"
        self._log_lines.append(line)

        # Update buffer
        new_text = "\n".join(self._log_lines)
        self._log_buffer.set_document(
            Document(text=new_text, cursor_position=len(new_text)),
            bypass_readonly=True,
        )

        # Trigger UI refresh
        self._app.invalidate()

    def _append_log_multiline(self, lines: str, prefix: str = "<<<") -> None:
        """Append multiple lines to the log with proper indentation."""
        timestamp = datetime.now().strftime("%H:%M:%S")
        first_line = True
        for line in lines.split("\n"):
            if first_line:
                self._log_lines.append(f"[{timestamp}] {prefix} {line}")
                first_line = False
            else:
                # Indent continuation lines
                indent = " " * (len(timestamp) + 3 + len(prefix) + 1)
                self._log_lines.append(f"{indent}{line}")

        # Update buffer
        new_text = "\n".join(self._log_lines)
        self._log_buffer.set_document(
            Document(text=new_text, cursor_position=len(new_text)),
            bypass_readonly=True,
        )

        # Trigger UI refresh
        self._app.invalidate()

    async def handle_snapshot(self, snapshot: AgentSnapshot) -> None:
        """Handle an incoming AgentSnapshot."""
        formatted = format_snapshot(snapshot)
        self._append_log_multiline(formatted, prefix="<<<")

    async def _handle_command(self, line: str) -> None:
        """Parse and handle a command."""
        # Log the command
        self._append_log(line, prefix=">>>")

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
        elif cmd in ("quit", "exit"):
            self._app.exit()
        else:
            self._append_log(f"Unknown command: {cmd}. Type 'help' for available commands.")

    def _print_help(self) -> None:
        """Print help message."""
        help_text = """Commands:
  help                          - Show this help message
  run <image> [tag] [port_map]  - Send run command
  stop <container_id>           - Send stop command
  status                        - Show connection status
  quit                          - Exit the CLI"""
        for line in help_text.split("\n"):
            self._append_log(line)

    async def _cmd_run(self, args: list[str]) -> None:
        """Handle the run command."""
        if not args:
            self._append_log("Usage: run <image> [tag] [port_map]")
            return

        connection = self._get_connection()
        if connection is None:
            self._append_log("Error: No agent connected")
            return

        image = args[0]
        tag = args[1] if len(args) > 1 else None
        port_mapping = args[2] if len(args) > 2 else None

        try:
            await connection.send_run_command(image, tag, port_mapping)
            self._append_log(f"Sent run command: image={image}, tag={tag or 'latest'}")
        except Exception as e:
            self._append_log(f"Error: {e}")

    async def _cmd_stop(self, args: list[str]) -> None:
        """Handle the stop command."""
        if not args:
            self._append_log("Usage: stop <container_id>")
            return

        connection = self._get_connection()
        if connection is None:
            self._append_log("Error: No agent connected")
            return

        container_id = args[0]

        try:
            await connection.send_stop_command(container_id)
            self._append_log(f"Sent stop command: container_id={container_id}")
        except Exception as e:
            self._append_log(f"Error: {e}")

    def _cmd_status(self) -> None:
        """Show connection status."""
        self._append_log(f"Status: {self._connection_status}")

    def _get_connection(self):
        """Get the current active connection, if any."""
        connections = self._server.connections
        for conn in connections:
            if not conn.is_closed:
                return conn
        return None

    def set_connection_status(self, status: str) -> None:
        """Update the connection status."""
        self._connection_status = status
        self._app.invalidate()

    async def run(self) -> None:
        """Run the TUI application."""
        await self._app.run_async()
