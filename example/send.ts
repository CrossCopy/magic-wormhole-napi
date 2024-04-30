import clipboard from "@crosscopy/clipboard";
import cliProgress from "cli-progress";

import { send } from "..";

if (process.argv.length !== 3) {
  console.error("Usage:  <file>");
  process.exit(1);
}
const file = process.argv.at(-1);
if (!file) {
  console.error("Invalid file");
  process.exit(1);
}
let start = Date.now();
const pb = new cliProgress.SingleBar({
  format:
    "Send Progress |{bar}| {percentage}% || {value}/{total} bytes || {speed}MB/s",
  barCompleteChar: "\u2588",
  barIncompleteChar: "\u2591",
  hideCursor: true,
});
await send(
  file,
  (err, code) => {
    console.log(`wormhole receive ${code}`);
    console.log("Written to clipboard");
    clipboard.setText(`${code}`);
  },
  (err, filesize) => {
    pb.start(Number(filesize), 0, {
      speed: "N/A",
    });
    start = Date.now();
  },
  (err, progress) => {
    let elapsed = (Date.now() - start) / 1000;
    const sent = Number(progress.sent);
    // sent is in bytes, compute MB/s
    pb.update(sent, { speed: (sent / 1_000_000 / elapsed).toFixed(2) });
  }
);
pb.stop();
