---
title: A first result
order: 20
evidence:
  - README.md:19-26 #5228e4e3469a
  - README.md:31-39 #79979b997ff3
  - src/main.rs:314-315 #580453feaa30
---

Install with Homebrew (`brew install lhwzds/tap/restloop`), npm (`npm install -g restloop`) or `cargo install --git https://github.com/lhwzds/restloop`. Only `RESTLOOP_MODEL` is required. The endpoint defaults to `https://api.openai.com/v1`, so for OpenAI a key and a model name are enough:

```bash
export RESTLOOP_API_KEY=sk-...
export RESTLOOP_MODEL=<model-id>
cd some-checkout
restloop "make the failing test pass"
```

For other providers set `RESTLOOP_BASE_URL` to the part before `/chat/completions`. Exit codes and every option are described in [[usage]].
