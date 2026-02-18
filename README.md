

<p align="center">
  <img src="ug2_banner.png" alt="ug2-client" />
</p>

# ug2-client

---

Downloads any Discord build, rewrites it with your branding and patches, and serves it as a fully working web client. Everything is configurable !


UG2 is a reimplementation of the Discord backend, it speaks the same API, same gateway, same everything.  
Any client/tool built for Discord works with UG2 out of the box: web, desktop, bots, third-party clients.

This repo handles the **web client** side: patching and serving it.

## Quick Start

**1.** Create a `.env` file (only `DATABASE_URL` is required, the rest has sane defaults):

```bash
DATABASE_URL=postgres://user:pass@localhost:5432/ug2-client
REDIS_URL=redis://127.0.0.1:6379
BIND_ADDR=0.0.0.0:3000
DISCORD_BASE_URL=https://discord.com
CACHE_PATH=./assets/cache
```

**2.** Configure your instance in `patch_config.toml`:

```toml
[branding]
instance_name = "My Instance"
instance_url = "https://canary.discord.com"
gateway_url = "wss://gateway.discord.gg"
sentry_url = "https://sentry.io"
status_url = "status.discord.com"

[patches]
nitro_rebranding = true
discord_rebranding = true
title_rebranding = true
server_to_guild = true
# ... see patch_config.toml for all options
```

**3.** Clone with submodules (the historical builds repo is included as a submodule):

```bash
git clone --recurse-submodules https://github.com/vbxq/ug2-client
```

If you already cloned without `--recurse-submodules`:

```bash
git submodule update --init
```

**4.** Build and run:

```bash
cargo run --release
```

**5.** Open `http://localhost:3000/selector`, pick a build, and enjoy.

### Importing historical builds

The repo includes [Discord-Build-Logger/Builds](https://github.com/Discord-Build-Logger/Builds) as a git submodule in `data/builds-repo/`. It contains archived Discord builds from 2022-2023.

To index them into your database:

```bash
cargo run -- import .\data\builds-repo\
```

You can also skip this and use the "Fetch Current Build" button in the selector to grab the latest live build directly.

Technically, I could support more years, but that would require more research, and UG2 doesn't need that right now.                                                                                                                                                                                

## Small API documentation

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/builds` | List all builds |
| `POST` | `/api/builds/download` | Download & patch a build (`{"build_hash": "..."}` or empty for latest) |
| `POST` | `/api/builds/fetch-current` | Scrape the current live Discord build directly |
| `PUT` | `/api/builds/active` | Set which build is served at `/` (`{"build_hash": "..."}`) |
| `PUT` | `/api/builds/{hash}/index-scripts` | Override entry scripts for a build |
| `POST` | `/api/builds/{hash}/repatch` | Re-apply patches to an already cached build |

When `api_proxy = true` (default), unmatched `/api/*` requests are proxied to Discord so the client works out of the box.  
PS: On a UG2 instance, you shouldn't need to enable api_proxy, that was needed for Discord to work with CORS problems.

## Patches

All patches are toggleable in `patch_config.toml`.

#### Branding
| Patch | Effect |
|-------|--------|
| `nitro_rebranding` | "Nitro" -> "Premium" |
| `discord_rebranding` | "Discord" -> your instance name |
| `title_rebranding` | Page title replacement |
| `server_to_guild` | "Server" -> "Guild" |

#### Infrastructure
| Patch | Effect |
|-------|--------|
| `sentry_redirect` | Redirect error reporting endpoint |
| `status_page_redirect` | Redirect status page URL |

#### Plugins
| Patch | Effect                                                        |
|-------|---------------------------------------------------------------|
| `prevent_localstorage_deletion` | Block localStorage wipe                                       |
| `fast_identify` | Skip fast-connect identify, use normal identify               |
| `gateway_reconnect` | Keep fast-connect mode on reconnect                           |
| `remove_qr_login` | Hide QR code login (doesn't work on newer build)              |
| `no_xss_warning` | Remove console self-XSS warning                               |
| `enable_dev_experiments` | Unlock developer experiments UI (seems broken, please verify) |
| `api_proxy` | Proxy `/api/*` to Discord (avoids CORS)                       |


## Rate Limiting

Optional per-IP rate limiting on `/api` routes, backed by Redis:

```toml
[server]
trust_proxy_headers = false  # set true behind Cloudflare/nginx to read real IPs
rate_limit_enabled = false
rate_limit_requests = 60     # max requests per window
rate_limit_window_secs = 60  # window duration
```

## TODO

- Authentication on the selector/API, right now anyone can access `/selector` and manage builds
- Make the patching system more robust (some patches break on newer builds)
- Support the developer portal page

---

<p align="center">
  <img src="ug2_selector.png" alt="Build selector" />
</p>
