import { useRef, useEffect, useCallback } from 'react';

export type RainState = 'idle' | 'scanning' | 'processing' | 'error' | 'off';

interface MatrixRainProps {
  rainState: RainState;
}

const KATAKANA = 'カキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヲン';
const DIGITS = '0123456789';
const SYMBOLS = '=*+-<>{}[]$#@&%';
const ALL_CHARS = KATAKANA + DIGITS + SYMBOLS;

const COL_WIDTH = 20;
const MAX_COLS = 80;
const TRAIL_LEN = 18;
const TARGET_FPS = 24;
const FRAME_INTERVAL = 1000 / TARGET_FPS;

interface Column {
  x: number;
  y: number;
  speed: number;
  chars: string[];
  opacity: number;
  isRed: boolean;
  redTimer: number;
}

function randomChar(): string {
  return ALL_CHARS[Math.floor(Math.random() * ALL_CHARS.length)];
}

function getStateConfig(state: RainState) {
  switch (state) {
    case 'idle':
      return { speedMin: 40, speedMax: 80, opacityMin: 0.15, opacityMax: 0.25, errorChance: 0 };
    case 'scanning':
      return { speedMin: 40, speedMax: 80, opacityMin: 0.25, opacityMax: 0.35, errorChance: 0 };
    case 'processing':
      return { speedMin: 100, speedMax: 200, opacityMin: 0.08, opacityMax: 0.12, errorChance: 0 };
    case 'error':
      return { speedMin: 40, speedMax: 80, opacityMin: 0.15, opacityMax: 0.25, errorChance: 0.02 };
    case 'off':
    default:
      return { speedMin: 0, speedMax: 0, opacityMin: 0, opacityMax: 0, errorChance: 0 };
  }
}

export function MatrixRain({ rainState }: MatrixRainProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const columnsRef = useRef<Column[]>([]);
  const rafRef = useRef<number>(0);
  const lastFrameRef = useRef<number>(0);
  const stateRef = useRef<RainState>(rainState);

  stateRef.current = rainState;

  const initColumns = useCallback((width: number, height: number) => {
    const numCols = Math.min(Math.floor(width / COL_WIDTH), MAX_COLS);
    const config = getStateConfig(stateRef.current);
    const cols: Column[] = [];

    for (let i = 0; i < numCols; i++) {
      const trail: string[] = [];
      for (let t = 0; t < TRAIL_LEN; t++) {
        trail.push(randomChar());
      }
      cols.push({
        x: i * COL_WIDTH + COL_WIDTH / 2,
        y: Math.random() * height * 1.5 - height * 0.5,
        speed: config.speedMin + Math.random() * (config.speedMax - config.speedMin),
        chars: trail,
        opacity: config.opacityMin + Math.random() * (config.opacityMax - config.opacityMin),
        isRed: false,
        redTimer: 0,
      });
    }
    columnsRef.current = cols;
  }, []);

  const render = useCallback((timestamp: number) => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const elapsed = timestamp - lastFrameRef.current;
    if (elapsed < FRAME_INTERVAL) {
      rafRef.current = requestAnimationFrame(render);
      return;
    }
    lastFrameRef.current = timestamp - (elapsed % FRAME_INTERVAL);

    const state = stateRef.current;
    if (state === 'off') {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      rafRef.current = requestAnimationFrame(render);
      return;
    }

    const config = getStateConfig(state);
    const dt = elapsed / 1000;
    const { width, height } = canvas;

    ctx.clearRect(0, 0, width, height);

    const cols = columnsRef.current;
    for (let i = 0; i < cols.length; i++) {
      const col = cols[i];

      // Lerp speed and opacity toward target state
      const targetSpeed = config.speedMin + Math.random() * (config.speedMax - config.speedMin);
      col.speed += (targetSpeed - col.speed) * 0.02;

      const targetOpacity = config.opacityMin + Math.random() * (config.opacityMax - config.opacityMin);
      col.opacity += (targetOpacity - col.opacity) * 0.02;

      // Move column down
      col.y += col.speed * dt;

      // Reset if off-screen
      if (col.y - TRAIL_LEN * 16 > height) {
        col.y = -TRAIL_LEN * 16;
        col.speed = config.speedMin + Math.random() * (config.speedMax - config.speedMin);
      }

      // Error state: randomly turn columns red
      if (config.errorChance > 0 && Math.random() < config.errorChance) {
        col.isRed = true;
        col.redTimer = 12;
      }
      if (col.redTimer > 0) {
        col.redTimer--;
        if (col.redTimer <= 0) col.isRed = false;
      }

      // 5% chance per frame a trail char changes
      if (Math.random() < 0.05) {
        const idx = Math.floor(Math.random() * col.chars.length);
        col.chars[idx] = randomChar();
      }

      // Draw trail
      ctx.font = '14px "IBM Plex Mono", monospace';
      for (let t = 0; t < TRAIL_LEN; t++) {
        const charY = col.y - t * 16;
        if (charY < -16 || charY > height + 16) continue;

        const fade = 1 - t / TRAIL_LEN;
        const alpha = col.opacity * fade;

        if (t === 0) {
          // Head char: brightest
          if (col.isRed) {
            ctx.fillStyle = `rgba(255, 0, 51, ${col.opacity * 1.5})`;
          } else {
            ctx.fillStyle = `rgba(0, 255, 65, ${col.opacity * 1.5})`;
          }
        } else {
          if (col.isRed) {
            ctx.fillStyle = `rgba(255, 0, 51, ${alpha})`;
          } else {
            ctx.fillStyle = `rgba(0, 255, 65, ${alpha})`;
          }
        }

        ctx.fillText(col.chars[t], col.x, charY);
      }
    }

    rafRef.current = requestAnimationFrame(render);
  }, []);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const resize = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
      initColumns(canvas.width, canvas.height);
    };

    resize();
    window.addEventListener('resize', resize);
    rafRef.current = requestAnimationFrame(render);

    const handleVisibility = () => {
      if (document.hidden) {
        cancelAnimationFrame(rafRef.current);
      } else {
        lastFrameRef.current = performance.now();
        rafRef.current = requestAnimationFrame(render);
      }
    };
    document.addEventListener('visibilitychange', handleVisibility);

    return () => {
      window.removeEventListener('resize', resize);
      document.removeEventListener('visibilitychange', handleVisibility);
      cancelAnimationFrame(rafRef.current);
    };
  }, [initColumns, render]);

  return (
    <canvas
      ref={canvasRef}
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        width: '100%',
        height: '100%',
        zIndex: 0,
        pointerEvents: 'none',
      }}
    />
  );
}
