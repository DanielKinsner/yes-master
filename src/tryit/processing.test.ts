import { describe, expect, it, vi } from "vitest";
import { PreviewWorker, type Job, type Reply } from "./processing";

const job = (intensity: number): Job => ({ kind: "render", start: 0, style: "universal", intensity, target: -11 });
function worker() {
  const port = { onmessage: null as ((event: { data: Reply }) => void) | null, onerror: null as (() => void) | null, postMessage: vi.fn(), terminate: vi.fn() };
  return { port, queue: new PreviewWorker(port as unknown as Worker), done: (id: number) => port.onmessage!({ data: { id, kind: "loaded", start: 0, peaks: new Float32Array(), duration: 30 } }) };
}
describe("preview worker queue", () => {
  it("keeps one active and only the latest pending job, discarding obsolete results", async () => {
    const { port, queue, done } = worker();
    const first = queue.request(job(0.1));
    const second = queue.request(job(0.2));
    const last = queue.request(job(0.9));
    expect(port.postMessage).toHaveBeenCalledTimes(1);
    await expect(second).resolves.toBeNull();
    done(1); await expect(first).resolves.toBeNull();
    expect(port.postMessage).toHaveBeenCalledTimes(2);
    expect(port.postMessage.mock.lastCall![0].intensity).toBe(0.9);
    done(3); await expect(last).resolves.toMatchObject({ id: 3 });
    queue.close();
  });
  it("invalidates running results immediately while a control is being debounced", async () => {
    const { queue, done } = worker();
    const first = queue.request(job(0.1));
    queue.invalidate(); done(1);
    await expect(first).resolves.toBeNull(); queue.close();
  });
  it("settles outstanding jobs on close and rejects future requests", async () => {
    const { queue, port } = worker();
    const pending = queue.request(job(0.1)); queue.close();
    await expect(pending).resolves.toBeNull();
    await expect(queue.request(job(0.2))).rejects.toThrow("closed");
    expect(port.terminate).toHaveBeenCalledOnce();
  });
  it("reports worker failures instead of leaving the interface processing forever", async () => {
    const { queue, port } = worker();
    const pending = queue.request(job(0.1));
    const rejected = expect(pending).rejects.toThrow("stopped");
    port.onerror!(); await rejected;
    await expect(queue.request(job(0.2))).rejects.toThrow("stopped");
  });
});
