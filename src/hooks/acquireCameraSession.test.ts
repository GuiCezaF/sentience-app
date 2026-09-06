import { afterEach, expect, mock, test } from "bun:test";
import {
  acquireCameraSession,
  peekCameraSession,
  resetCameraSessionForTests,
} from "./acquireCameraSession";

function fakeStream(id: string): MediaStream {
  return {
    id,
    getTracks: () => [{ stop: mock(() => {}) }],
  } as unknown as MediaStream;
}

afterEach(() => {
  resetCameraSessionForTests();
});

test("chamadas concorrentes disparam getUserMedia uma vez só", async () => {
  let calls = 0;
  const media = fakeStream("cam-1");
  const devices = {
    getUserMedia: mock(() => {
      calls += 1;
      return new Promise<MediaStream>((resolve) => {
        setTimeout(() => resolve(media), 5);
      });
    }),
  };

  const [a, b] = await Promise.all([
    acquireCameraSession(devices),
    acquireCameraSession(devices),
  ]);

  expect(calls).toBe(1);
  expect(a).toBe(media);
  expect(b).toBe(media);
  expect(peekCameraSession()).toBe(media);
});

test("depois de aberta, a sessão reusa o stream sem novo pedido", async () => {
  const first = fakeStream("cam-1");
  const devices = {
    getUserMedia: mock(() => Promise.resolve(first)),
  };

  await acquireCameraSession(devices);
  const again = await acquireCameraSession({
    getUserMedia: mock(() => Promise.resolve(fakeStream("cam-2"))),
  });

  expect(devices.getUserMedia).toHaveBeenCalledTimes(1);
  expect(again).toBe(first);
});

test("sem getUserMedia a aquisição falha e não deixa sessão pela metade", async () => {
  await expect(acquireCameraSession(undefined)).rejects.toThrow("getUserMedia");
  expect(peekCameraSession()).toBeNull();
});
