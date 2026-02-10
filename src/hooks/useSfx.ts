import { useCallback, useRef } from 'react';

let audioCtx: AudioContext | null = null;

function getCtx(): AudioContext {
  if (!audioCtx) audioCtx = new AudioContext();
  return audioCtx;
}

/**
 * Filtered mouse click — noise impulse through resonant filter.
 * Like a real click but with tonal color. ~40ms.
 */
function playHoverClick() {
  const ctx = getCtx();
  const now = ctx.currentTime;

  // Short noise impulse — the "click" body
  const len = Math.floor(ctx.sampleRate * 0.005); // 5ms
  const buffer = ctx.createBuffer(1, len, ctx.sampleRate);
  const data = buffer.getChannelData(0);
  for (let i = 0; i < len; i++) {
    // Sharp impulse with instant decay
    data[i] = (Math.random() * 2 - 1) * Math.pow(1 - i / len, 6);
  }

  const src = ctx.createBufferSource();
  src.buffer = buffer;

  // Resonant filter — gives the click its "color"
  const filter = ctx.createBiquadFilter();
  filter.type = 'bandpass';
  filter.frequency.setValueAtTime(3200, now);
  filter.frequency.exponentialRampToValueAtTime(1200, now + 0.04);
  filter.Q.value = 12; // high resonance = ringing tail from the click

  const gain = ctx.createGain();
  gain.gain.setValueAtTime(0.8, now);
  gain.gain.exponentialRampToValueAtTime(0.001, now + 0.04);

  src.connect(filter).connect(gain).connect(ctx.destination);
  src.start(now);
}

/**
 * Click: same idea but lower, heavier, double-filtered.
 * Two filter bands for richer "thock". ~60ms.
 */
function playButtonClick() {
  const ctx = getCtx();
  const now = ctx.currentTime;

  const len = Math.floor(ctx.sampleRate * 0.006);
  const buffer = ctx.createBuffer(1, len, ctx.sampleRate);
  const data = buffer.getChannelData(0);
  for (let i = 0; i < len; i++) {
    data[i] = (Math.random() * 2 - 1) * Math.pow(1 - i / len, 5);
  }

  // High band — the "tick"
  const src1 = ctx.createBufferSource();
  src1.buffer = buffer;
  const f1 = ctx.createBiquadFilter();
  f1.type = 'bandpass';
  f1.frequency.setValueAtTime(4000, now);
  f1.frequency.exponentialRampToValueAtTime(1800, now + 0.04);
  f1.Q.value = 10;
  const g1 = ctx.createGain();
  g1.gain.setValueAtTime(0.6, now);
  g1.gain.exponentialRampToValueAtTime(0.001, now + 0.04);
  src1.connect(f1).connect(g1).connect(ctx.destination);
  src1.start(now);

  // Low band — the "thock" body
  const src2 = ctx.createBufferSource();
  src2.buffer = buffer;
  const f2 = ctx.createBiquadFilter();
  f2.type = 'bandpass';
  f2.frequency.setValueAtTime(800, now);
  f2.frequency.exponentialRampToValueAtTime(400, now + 0.06);
  f2.Q.value = 6;
  const g2 = ctx.createGain();
  g2.gain.setValueAtTime(0.7, now);
  g2.gain.exponentialRampToValueAtTime(0.001, now + 0.06);
  src2.connect(f2).connect(g2).connect(ctx.destination);
  src2.start(now);
}

/**
 * Action button — deeper, longer, triple-band hit.
 * For primary CTA buttons (CONNECT, SCAN, BEGIN WRITE, etc.). ~100ms.
 */
function playActionClick() {
  const ctx = getCtx();
  const now = ctx.currentTime;

  const len = Math.floor(ctx.sampleRate * 0.008);
  const buffer = ctx.createBuffer(1, len, ctx.sampleRate);
  const data = buffer.getChannelData(0);
  for (let i = 0; i < len; i++) {
    data[i] = (Math.random() * 2 - 1) * Math.pow(1 - i / len, 4);
  }

  // High ping — attack transient
  const src1 = ctx.createBufferSource();
  src1.buffer = buffer;
  const f1 = ctx.createBiquadFilter();
  f1.type = 'bandpass';
  f1.frequency.setValueAtTime(5000, now);
  f1.frequency.exponentialRampToValueAtTime(2000, now + 0.05);
  f1.Q.value = 14;
  const g1 = ctx.createGain();
  g1.gain.setValueAtTime(0.5, now);
  g1.gain.exponentialRampToValueAtTime(0.001, now + 0.05);
  src1.connect(f1).connect(g1).connect(ctx.destination);
  src1.start(now);

  // Mid ring — tonal body
  const src2 = ctx.createBufferSource();
  src2.buffer = buffer;
  const f2 = ctx.createBiquadFilter();
  f2.type = 'bandpass';
  f2.frequency.setValueAtTime(1600, now);
  f2.frequency.exponentialRampToValueAtTime(900, now + 0.1);
  f2.Q.value = 16;
  const g2 = ctx.createGain();
  g2.gain.setValueAtTime(0.7, now);
  g2.gain.exponentialRampToValueAtTime(0.001, now + 0.1);
  src2.connect(f2).connect(g2).connect(ctx.destination);
  src2.start(now);

  // Sub thump — weight
  const src3 = ctx.createBufferSource();
  src3.buffer = buffer;
  const f3 = ctx.createBiquadFilter();
  f3.type = 'lowpass';
  f3.frequency.setValueAtTime(500, now);
  f3.frequency.exponentialRampToValueAtTime(200, now + 0.08);
  f3.Q.value = 4;
  const g3 = ctx.createGain();
  g3.gain.setValueAtTime(0.6, now);
  g3.gain.exponentialRampToValueAtTime(0.001, now + 0.08);
  src3.connect(f3).connect(g3).connect(ctx.destination);
  src3.start(now);
}

export function useSfx() {
  const lastHoverRef = useRef(0);

  const hover = useCallback(() => {
    const now = Date.now();
    if (now - lastHoverRef.current < 80) return;
    lastHoverRef.current = now;
    playHoverClick();
  }, []);

  const click = useCallback(() => {
    playButtonClick();
  }, []);

  const action = useCallback(() => {
    playActionClick();
  }, []);

  return { hover, click, action };
}
