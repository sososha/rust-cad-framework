# Rust CAD Framework 🦀📐

A high-performance, agent-ready CAD application framework written in Rust.
Designed to be the perfect starting point for building modern CAD tools with AI assistance in mind.

## Features

*   **🚀 High Performance**: Built on `wgpu` for modern, cross-platform GPU rendering.
*   **♾️ Infinite Canvas**: Built-in support for panning (Middle Click / Space + Drag) and zooming (Wheel / Pinch).
*   **🤖 Agent Ready**: Embedded IPC server allows AI agents to control the application, inspect state, and run tests automatically.
*   **🛠️ Extensible Tool System**: Easy-to-implement `Tool` trait for adding new drawing capabilities.
*   **🏗️ Solid Architecture**: Clean separation of concerns (App, Canvas, Geometry, Tools).

## Quick Start

### Prerequisites
*   Rust (latest stable)

### Running the App

```bash
git clone https://github.com/your-username/rust-cad-framework.git
cd rust-cad-framework
cargo run
```

## AI Agent Interface

This framework includes a unique "Agent Interface" running on `localhost:9000`.
This allows you to write scripts or use AI agents to control the CAD software programmatically.

**Example: Draw a line via API**

```bash
curl -X POST http://localhost:9000/api/command \
  -H "Content-Type: application/json" \
  -d '{
    "action": "draw_line",
    "args": { "start": [0, 0], "end": [500, 500] }
  }'
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License
