/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export function helloWorld(): string
export interface ProgressHandlerPayload {
  sent: bigint
  total: bigint
}
export function send(filepath: string, codeCallback: (err: Error | null, arg: string) => any, startCallback: (err: Error | null, arg: bigint) => any, progressCallback: (err: Error | null, arg: ProgressHandlerPayload) => any): Promise<void>
export function receive(code: string, outputDir: string, startCallback: (err: Error | null, arg: bigint) => any, progressCallback: (err: Error | null, arg: ProgressHandlerPayload) => any): Promise<void>
export function callThreadsafeFunction(tsfn: (err: Error | null, arg: number) => any): void
