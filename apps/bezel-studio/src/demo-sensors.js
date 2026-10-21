/** Well-known demo sensors: key, category, label, quantity, base value, swing. */
export const DEMO_SENSORS = Object.freeze([
  ['cpu.usage', 'cpu', 'CPU usage', 'percent', 32, 20],
  ['cpu.temperature', 'cpu', 'CPU temperature', 'celsius', 52, 8],
  ['cpu.frequency', 'cpu', 'CPU frequency', 'megahertz', 4720, 300],
  ['cpu.power', 'cpu', 'CPU power', 'watts', 64, 20],
  ['gpu.usage', 'gpu', 'GPU usage', 'percent', 41, 30],
  ['gpu.temperature', 'gpu', 'GPU temperature', 'celsius', 47, 6],
  ['gpu.power', 'gpu', 'GPU power', 'watts', 180, 60],
  ['memory.percent', 'memory', 'Memory in use', 'percent', 38, 3],
  ['memory.used', 'memory', 'Memory used', 'bytes', 24e9, 1e9],
  ['net.down', 'network', 'Download', 'bytesPerSecond', 2.4e6, 2e6],
  ['net.up', 'network', 'Upload', 'bytesPerSecond', 3.1e5, 2e5],
  ['disk.read', 'disk', 'Disk read', 'bytesPerSecond', 8e6, 7e6],
  ['hwmon.nvme0.composite', 'board', 'NVMe composite', 'celsius', 40, 2],
  ['system.volume', 'system', 'Output volume', 'percent', 48, 0],
  ['system.audio.muted', 'system', 'Output muted', 'number', 0, 0],
  ['system.power.ac', 'system', 'Connected to power', 'number', 1, 0],
  ['system.uptime', 'system', 'Uptime', 'seconds', 93784, 0],
  ['cpu.fan', 'cpu', 'CPU fan', 'rpm', 1200, 100],
  ['gpu.memory.used', 'gpu', 'GPU memory used', 'bytes', 2701 * 1024 ** 2, 64 * 1024 ** 2],
  ['memory.total', 'memory', 'Memory total', 'bytes', 64 * 1024 ** 3, 0],
  ['memory.available.percent', 'memory', 'Memory available', 'percent', 62, 0],
]);

/** Sensor value at time t (seconds): a smooth wobble around the base. */
export function demoValue(base, swing, t, seed) {
  return Math.max(0, base + swing * Math.sin(t / 3 + seed) * 0.8 + swing * 0.2 * Math.sin(t * 1.7 + seed * 2));
}

/** Native demo reading, or null for an unknown/unavailable sensor. */
export const DEMO_SYSTEM_TEXT = Object.freeze({ 'cpu.name': 'Demo CPU 12-Core Processor', 'gpu.name': 'Demo GPU', 'system.hostname': 'bezel-demo-workstation', 'system.os': 'Windows 11', 'system.audio.device': 'Demo headphones' });

/** The existing current-weather fixture, shared by Weather and bound values. */
export const DEMO_WEATHER = Object.freeze({ temperature: 20, code: 3 });

export function demoSensorValue(key, t) {
  if (key === 'memory.available.percent') return 100 - demoSensorValue('memory.percent', t);
  const weather = String(key).match(/^weather\.(-?\d+(?:\.\d+)?):(-?\d+(?:\.\d+)?)\.(temperature|code)$/);
  if (weather && Math.abs(Number(weather[1])) <= 90 && Math.abs(Number(weather[2])) <= 180) return DEMO_WEATHER[weather[3]];
  const index = DEMO_SENSORS.findIndex(row => row[0] === key);
  if (index < 0) return null;
  const row = DEMO_SENSORS[index];
  return demoValue(row[4], row[5], t, index);
}
