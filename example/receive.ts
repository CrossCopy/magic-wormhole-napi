import clipboard from "@crosscopy/clipboard";
import cliProgress from "cli-progress";

import { receive } from "..";

if (process.argv.length !== 3) {
  console.error("Usage:  <code>");
  process.exit(1);
}
const code = process.argv.at(-1);
if (!code) {
  console.error("Invalid code");
  process.exit(1);
}

const pb = new cliProgress.SingleBar({
  format:
    "Receive Progress |{bar}| {percentage}% || {value}/{total} bytes || {speed}MB/s",
  barCompleteChar: "\u2588",
  barIncompleteChar: "\u2591",
  hideCursor: true,
});
let start = Date.now();

await receive(
  code,
  process.cwd(),
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
