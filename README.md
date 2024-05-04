# magic-wormhole-napi

<div>
  <a href="https://github.com/CrossCopy/magic-wormhole-napi/actions/workflows/CI.yml"><img alt="CI Badge" src="https://github.com/CrossCopy/magic-wormhole-napi/actions/workflows/CI.yml/badge.svg" /></a>
  <a href="https://www.npmjs.com/package/magic-wormhole-napi"><img alt="NPM Downloads" src="https://img.shields.io/npm/v/magic-wormhole-napi"></a>
  
  <a href="https://www.npmjs.com/package/magic-wormhole-napi"><img alt="NPM Downloads" src="https://img.shields.io/npm/dm/magic-wormhole-napi"></a>
</div>

NPM Package: https://www.npmjs.com/package/magic-wormhole-napi

This is a Node.js native addon for [Magic Wormhole](https://github.com/magic-wormhole/magic-wormhole.rs.git)

The original implementation is in Rust based on [magic-wormhole.rs](https://github.com/magic-wormhole/magic-wormhole.rs), [napi-rs](https://napi.rs/) was used to create the Node.js native addon.

This project is still under early development, the APIs support the most basic functionalities of Magic Wormhole.

## Supported Commands

- `send`
- `receive`

## Usage

```ts
import { send, receive } from "magic-wormhole-napi";

await send(
  file, // file path in string
  (err, code) => {
    // wormhole code callback
    console.log(`wormhole receive ${code}`);
  },
  (err, filesize) => {
    // start receiving callback
    console.log(`Start Sending ${filesize} bytes`);
  },
  (err, progress) => {
    // progress update callback
    const { sent, total } = progress;
  }
);

await receive(
  code,
  process.cwd(), // where to save the file
  (err, filesize) => {
    // start receiving callback
  },
  (err, progress) => {
    // progress update callback
    const { sent, total } = progress;
  }
);
```

## Development

```bash
git clone https://github.com/CrossCopy/magic-wormhole-napi --recursive # there is a submodule required to build
yarn        # install dependencies
yarn build
```

## Example

See [./example](./example) for more details. This is an example that turns the napi library into an CLI.

There is a [sent.ts](./example/send.ts) and [receive.ts](./example/receive.ts) example.

```bash
cd example
bun install
bun send.ts ~/Desktop/video.mp4
bun receive.ts 1-nice-idea
```

## Publish

```bash
npm version patch
git push --follow-tags
```

A normal commit won't trigger publish. `git log -1 --pretty=%B | grep "^[0-9]\+\.[0-9]\+\.[0-9]\+$"` is used to check if the commit message is a version number. So always add a tag to the commit or have the pattern in the commit message.
