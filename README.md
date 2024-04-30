# magic-wormhole-napi

This is a Node.js native addon for [Magic Wormhole](https://github.com/magic-wormhole/magic-wormhole.rs.git)

The original implementation is in Rust, [napi-rs](https://napi.rs/) was used to create the Node.js native addon.

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

## Example

See [./example](./example) for more details. This is an example that turns the napi library into an CLI.

There is a [sent.ts](./example/send.ts) and [receive.ts](./example/receive.ts) example.

```bash
bun send.ts ~/Desktop/video.mp4
bun receive.ts 1-nice-idea
```
