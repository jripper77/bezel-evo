// Text value parts, matching bezel-core/domain/sensor.rs::format_reading_parts.
export function formatValue(value, quantity, format = {}) {
  if (typeof value === 'string') return { number: value, unit: '' };
  if (!Number.isFinite(value)) return { number: '—', unit: '' };
  let number = value, unit = '', decimals = 1;
  if (quantity === 'seconds') {
    const total = Math.floor(Math.max(0, value)), days = Math.floor(total / 86400), hours = Math.floor(total % 86400 / 3600), minutes = Math.floor(total % 3600 / 60);
    return { number: `${days ? `${days}d ` : ''}${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}`, unit: '' };
  }
  if (quantity === 'bytes' || quantity === 'bytesPerSecond') {
    const base = format.decimalBytes ? 1000 : 1024, names = format.decimalBytes ? ['B','kB','MB','GB','TB'] : ['B','KiB','MiB','GiB','TiB'];
    number = Math.max(0, value); let index = 0;
    while (number >= base && index < 4) { number /= base; index++; }
    unit = ` ${names[index]}${quantity === 'bytesPerSecond' ? '/s' : ''}`;
    decimals = index === 0 || number >= 100 ? 0 : 1;
  } else if (quantity === 'megahertz') {
    if (value >= 1000) { number /= 1000; unit = ' GHz'; decimals = 2; }
    else { unit = ' MHz'; decimals = 0; }
  } else {
    const spec = { percent: ['%',0], celsius: [format.fahrenheit ? '°F' : '°C',0], watts: [' W',0], volts: [' V',2], amperes: [' A',2], rpm: [' RPM',0] }[quantity];
    if (spec) [unit, decimals] = spec;
    if (quantity === 'celsius' && format.fahrenheit) number = number * 9 / 5 + 32;
  }
  return { number: number.toFixed(Math.min(9, Math.max(0, format.decimals ?? decimals))), unit: format.showUnit === false ? '' : unit };
}
