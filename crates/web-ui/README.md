# ARM Hypervisor Web UI

React-based web interface for the ARM Hypervisor Platform.

## Development

```bash
cd crates/web-ui
npm install
npm run dev
```

The development server will start on `http://localhost:3000` and proxy API requests to `http://localhost:8080`.

## Building

```bash
npm run build
```

The built files will be in the `dist/` directory and can be served statically or integrated into the API server.

## Features

- Dashboard with cluster and container overview
- Container management (create, start, stop, delete)
- Cluster node management
- Storage pool management
- Network bridge management
