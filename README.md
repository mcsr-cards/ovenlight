# ovenlight

Image classifier for [mcsr.cards](https://mcsr.cards).

Runs [nsfwjs](https://github.com/infinitered/nsfwjs) MobileNetV2 model through [tract](https://github.com/sonos/tract).

## Usage

```
POST /
content-type: image/webp
authorization: Bearer <NSFW_TOKEN>      (only checked when NSFW_TOKEN is set)
body: the image bytes
```

Example Result
```json
{ "Drawing": 0.02, "Hentai": 0.005, "Neutral": 0.87, "Porn": 0.09, "Sexy": 0.004 }
```

| status | meaning |
| --- | --- |
| 401 | missing or wrong bearer token |
| 422 | body is not a decodable image |
| 500 | inference failed |

Request bodies are capped at 8 MB.

## Run

| variable | default | |
| --- | --- | --- |
| `NSFW_TOKEN` | unset | bearer token required on requests; unset means the endpoint is open |
| `HOST` | `0.0.0.0` | address to bind, use `127.0.0.1` for localhost only |
| `PORT` | `3003` | port to listen on |

```
NSFW_TOKEN=secret cargo run --release
```

You can provide multiple tokens, separated by commas: `NSFW_TOKEN=skibidi,sigma`

Generate with `openssl rand -hex 32`

### Docker

```
docker run -d --name ovenlight --restart unless-stopped -p 127.0.0.1:3003:3003 -e NSFW_TOKEN=secret ghcr.io/mcsr-cards/ovenlight
```

Or if you want to build it yourself like a chud loser: `docker build -t ovenlight .`
