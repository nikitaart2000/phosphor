import { useState, useEffect, useCallback } from 'react';

const BASE_TEXT = 'nik shuv';
const GLITCH_CHARS = '@#$%&*!?/\\|~^';

function glitchText(original: string): string {
  const chars = original.split('');
  const numSwaps = 1 + Math.floor(Math.random() * 3);
  for (let i = 0; i < numSwaps; i++) {
    const idx = Math.floor(Math.random() * chars.length);
    chars[idx] = GLITCH_CHARS[Math.floor(Math.random() * GLITCH_CHARS.length)];
  }
  return chars.join('');
}

export function GlitchWatermark() {
  const [text, setText] = useState(BASE_TEXT);
  const [isGlitching, setIsGlitching] = useState(false);

  const triggerGlitch = useCallback(() => {
    setIsGlitching(true);
    let count = 0;
    const maxFrames = 4 + Math.floor(Math.random() * 4);

    const glitchInterval = setInterval(() => {
      setText(glitchText(BASE_TEXT));
      count++;
      if (count >= maxFrames) {
        clearInterval(glitchInterval);
        setText(BASE_TEXT);
        setIsGlitching(false);
      }
    }, 60);
  }, []);

  useEffect(() => {
    const scheduleNext = () => {
      const delay = 3000 + Math.random() * 5000; // 3-8 seconds
      return setTimeout(() => {
        triggerGlitch();
        timerRef = scheduleNext();
      }, delay);
    };

    let timerRef = scheduleNext();
    return () => clearTimeout(timerRef);
  }, [triggerGlitch]);

  // RGB channel split offsets for glitch effect
  const redOffset = isGlitching ? `${-1 + Math.random() * 2}px` : '0';
  const blueOffset = isGlitching ? `${-1 + Math.random() * 2}px` : '0';

  return (
    <div
      style={{
        position: 'fixed',
        bottom: '6px',
        right: '8px',
        fontFamily: 'var(--font-mono)',
        fontSize: '11px',
        color: 'var(--green-ghost)',
        pointerEvents: 'none',
        zIndex: 9998,
        userSelect: 'none',
        textShadow: isGlitching
          ? `${redOffset} 0 rgba(255,0,51,0.5), ${blueOffset} 0 rgba(0,153,255,0.5)`
          : 'none',
        opacity: isGlitching ? 0.6 : 0.3,
        transition: 'opacity 0.05s',
      }}
    >
      {text}
    </div>
  );
}
