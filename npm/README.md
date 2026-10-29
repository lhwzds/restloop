# restloop

A minimal coding-agent loop: it keeps working on your task so you can rest.

This package runs the [RestLoop](https://github.com/lhwzds/restloop) binary. On first use it
downloads the binary for your platform (macOS, Linux x86_64 and arm64, Windows x86_64) from the GitHub
release and checks its SHA-256.

```bash
npm install -g restloop
export RESTLOOP_API_KEY=... RESTLOOP_MODEL=...
restloop "fix the failing test in this repo"
```

See the [project README](https://github.com/lhwzds/restloop) for all options.
