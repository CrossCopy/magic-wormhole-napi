import clipboard from "@crosscopy/clipboard";
import cliProgress from "cli-progress";

import { send } from "..";

const pb = new cliProgress.SingleBar({
  format:
    "CLI Progress |{bar}| {percentage}% || {value}/{total} bytes || Speed: {speed}",
  barCompleteChar: "\u2588",
  barIncompleteChar: "\u2591",
  hideCursor: true,
});
await send(
  "/Users/hacker/Movies/tmp/niagara-drone_1.mp4",
  (err, code) => {
    console.log(`wormhole receive ${code}`);
    console.log("Written to clipboard");
    clipboard.setText(`wormhole receive ${code}`);
  },
  (err, filesize) => {
    pb.start(Number(filesize), 0, {
      speed: "N/A",
    });
  },
  (err, progress) => {
    pb.update(Number(progress.sent));
    // pb.increment(Number(progress.sent));
  }
);
pb.stop();
