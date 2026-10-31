---
id: simkit-api-client
title: API Client Module
custom_edit_url: https://github.com/gemphi/phi/edit/main/crates/simkit/src/api/client/README.md
---

# SimKit API Client

This module contains the `RpcClient` implementation which uses `msgpack-rpc` to communicate with the simulation server.

## Structure

Due to the size of the API, the client is broken down into multiple submodules:

*   **`base.rs`**: Core client connection, setup, runtime management, and generic server RPCs.
*   **`sensors.rs`**: Endpoints for retrieving sensor data (IMU, GPS, Lidar, Barometer, etc.).
*   **`vehicles.rs`**: Endpoints for controlling and retrieving the state of vehicles (Cars and Multirotors).
*   **`vision.rs`**: Endpoints for cameras and image capture pipelines.
*   **`scene.rs`**: Endpoints for scene manipulation, weather, time of day, and segmentation objects.

## Usage

```rust
use simkit::api::RpcClient;

let mut client = RpcClient::new("localhost");
client.connect().await?;
```
