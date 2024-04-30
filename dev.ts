import { send, callThreadsafeFunction } from ".";

await send(
  "/Users/hacker/Movies/tmp/niagara-drone_1.mp4",
  (err, code) => {
    console.log(`wormhole receive ${code}`);
  },
  (err, progress) => {
    console.log(`sent: ${progress.sent}, total: ${progress.total}`);
  }
);
// callThreadsafeFunction((err, v) => {
//   console.log(err, v);
// });
