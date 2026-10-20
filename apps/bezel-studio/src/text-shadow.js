// Same transparent-edge separable box filter as bezel-render/text/shadow.rs.
export function blurMask(data, width, height, radius) {
  const r = Math.max(0, Math.min(64, Math.round(radius))), divisor = r * 2 + 1;
  const horizontal = new Uint8Array(data.length), result = new Uint8Array(data.length);
  for (let y = 0; y < height; y++) {
    let sum = 0;
    for (let x = 0; x <= Math.min(r, width - 1); x++) sum += data[y * width + x];
    for (let x = 0; x < width; x++) {
      horizontal[y * width + x] = Math.floor(sum / divisor);
      if (x >= r) sum -= data[y * width + x - r];
      if (x + r + 1 < width) sum += data[y * width + x + r + 1];
    }
  }
  for (let x = 0; x < width; x++) {
    let sum = 0;
    for (let y = 0; y <= Math.min(r, height - 1); y++) sum += horizontal[y * width + x];
    for (let y = 0; y < height; y++) {
      result[y * width + x] = Math.floor(sum / divisor);
      if (y >= r) sum -= horizontal[(y - r) * width + x];
      if (y + r + 1 < height) sum += horizontal[(y + r + 1) * width + x];
    }
  }
  return result;
}

/** Draw a centred shadow inside the element's clip, without affecting text layout. */
export function drawTextShadow(ctx, style, frame, draw) {
  if (!style.shadow || frame.width <= 0 || frame.height <= 0) return;
  const width = Math.ceil(frame.width), height = Math.ceil(frame.height);
  const canvas = new OffscreenCanvas(width, height), mask = canvas.getContext('2d');
  mask.translate(-frame.x, -frame.y);
  const parts = Object.fromEntries(Object.entries(style.parts ?? {}).map(([key, part]) => [key, part ? { ...part, color: '#ffffffff' } : part]));
  draw(mask, { ...style, paint: '#ffffffff', parts, shadow: null });
  const image = mask.getImageData(0, 0, width, height);
  const data = image.data, alpha = blurMask(Uint8Array.from({ length: width * height }, (_, i) => data[i * 4 + 3]), width, height, style.shadow.blur);
  mask.resetTransform();
  mask.clearRect(0, 0, width, height);
  for (let i = 0; i < alpha.length; i++) { data[i * 4] = data[i * 4 + 1] = data[i * 4 + 2] = 255; data[i * 4 + 3] = alpha[i]; }
  mask.putImageData(image, 0, 0);
  mask.globalCompositeOperation = 'source-in';
  mask.fillStyle = style.shadow.color;
  mask.fillRect(0, 0, width, height);
  ctx.drawImage(canvas, frame.x, frame.y);
}
