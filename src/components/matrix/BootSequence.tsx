import { useState, useEffect, useCallback } from 'react';
import { useTypewriter } from '../../hooks/useTypewriter';

interface BootSequenceProps {
  onComplete: () => void;
}

const GARBLE = '!@#$%&*<>{}|=+-~0123456789ABCDEF';

interface BootLine {
  text: string;
  color: string;
  decodeMs?: number;
}

const BOOT_LINES: BootLine[] = [
  { text: '[=] POST..................... PHOSPHOR SYSTEMS v1.1.0', color: 'var(--green-dim)' },
  { text: '[+] Card database............ 34 types loaded', color: 'var(--green-mid)' },
  { text: '[+] RF antenna............... calibrated', color: 'var(--green-mid)' },
  { text: '[+] USB...................... ports enumerated', color: 'var(--green-mid)' },
  { text: '[-] FBI...................... not detected (you\'re welcome)', color: 'var(--green-dim)' },
  { text: '[!] Coffee................... DANGEROUSLY LOW', color: 'var(--amber)' },
  { text: '[-] Ethics module............ [NOT INSTALLED]', color: 'var(--green-dim)' },
  { text: '[+] Plausible deniability.... enabled', color: 'var(--green-mid)' },
  { text: '[!] For educational and authorized use only', color: 'var(--amber)', decodeMs: 800 },
];

const LINE_INTERVAL = 300;
const DECODE_MS = 200;
const TITLE_START = 400;
const LINES_START = 2200;
const ONLINE_TIME = LINES_START + BOOT_LINES.length * LINE_INTERVAL + 900;
const COMPLETE_TIME = ONLINE_TIME + 1000;

// Characters that keep their shape during decode (visual structure)
function isStructural(ch: string): boolean {
  return ch === ' ' || ch === '.' || ch === '[' || ch === ']';
}

export function BootSequence({ onComplete }: BootSequenceProps) {
  const [phase, setPhase] = useState(1);
  const [lineStarts, setLineStarts] = useState<(number | null)[]>(
    BOOT_LINES.map(() => null)
  );
  const [, setTick] = useState(0);
  const [cursorVisible, setCursorVisible] = useState(true);

  const titleText = phase >= 2 ? 'PHOSPHOR' : '';
  const displayedTitle = useTypewriter(titleText, 55);
  const subtitleText = phase >= 2 ? 'v1.1.0 // PROXMARK3 INTERFACE' : '';
  const displayedSubtitle = useTypewriter(
    displayedTitle === titleText ? subtitleText : '',
    18,
  );

  const skip = useCallback(() => {
    setPhase(5);
    onComplete();
  }, [onComplete]);

  // Cursor blink for phase 1
  useEffect(() => {
    if (phase !== 1) return;
    const t = setInterval(() => setCursorVisible(v => !v), 530);
    return () => clearInterval(t);
  }, [phase]);

  // Phase transitions
  useEffect(() => {
    if (phase >= 5) return;
    const timers: ReturnType<typeof setTimeout>[] = [];

    timers.push(setTimeout(() => setPhase(2), TITLE_START));
    timers.push(setTimeout(() => setPhase(3), LINES_START));

    BOOT_LINES.forEach((_, i) => {
      timers.push(
        setTimeout(() => {
          setLineStarts(prev => {
            const next = [...prev];
            next[i] = Date.now();
            return next;
          });
        }, LINES_START + i * LINE_INTERVAL),
      );
    });

    timers.push(setTimeout(() => setPhase(4), ONLINE_TIME));
    timers.push(
      setTimeout(() => {
        setPhase(5);
        onComplete();
      }, COMPLETE_TIME),
    );

    return () => timers.forEach(clearTimeout);
  }, [onComplete, phase]);

  // Scramble animation tick (~30fps during boot lines)
  useEffect(() => {
    if (phase < 3 || phase >= 5) return;
    const t = setInterval(() => setTick(n => n + 1), 33);
    return () => clearInterval(t);
  }, [phase]);

  // Skip on click or keypress
  useEffect(() => {
    const handler = () => skip();
    window.addEventListener('click', handler);
    window.addEventListener('keydown', handler);
    return () => {
      window.removeEventListener('click', handler);
      window.removeEventListener('keydown', handler);
    };
  }, [skip]);

  if (phase >= 5) return null;

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 100,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'var(--bg-void)',
        fontFamily: 'var(--font-mono)',
        color: 'var(--green-bright)',
      }}
    >
      {/* Phase 1: blinking cursor */}
      {phase === 1 && (
        <div style={{ fontSize: '20px', opacity: cursorVisible ? 1 : 0 }}>_</div>
      )}

      {/* Phase 2+: Title with glow */}
      {phase >= 2 && (
        <div style={{ textAlign: 'center', marginBottom: phase >= 3 ? '32px' : 0 }}>
          <div
            style={{
              fontSize: '48px',
              fontWeight: 700,
              letterSpacing: '8px',
              textShadow:
                '0 0 10px rgba(0,255,65,0.5), 0 0 20px rgba(0,255,65,0.3), 0 0 40px rgba(0,255,65,0.15)',
            }}
          >
            {displayedTitle}
            {displayedTitle !== titleText && (
              <span style={{ opacity: 0.7 }}>_</span>
            )}
          </div>
          <div
            style={{
              fontSize: '13px',
              color: 'var(--green-dim)',
              marginTop: '8px',
              letterSpacing: '3px',
            }}
          >
            {displayedSubtitle}
            {displayedSubtitle !== subtitleText &&
              displayedTitle === titleText && (
                <span style={{ opacity: 0.7 }}>_</span>
              )}
          </div>
        </div>
      )}

      {/* Phase 3+: Boot lines with scramble-decode */}
      {phase >= 3 && (
        <div
          style={{
            textAlign: 'left',
            fontSize: '13px',
            lineHeight: '1.8',
            maxWidth: '520px',
            width: '90%',
          }}
        >
          {BOOT_LINES.map((line, i) => {
            const start = lineStarts[i];
            if (start === null) return null;
            const elapsed = Date.now() - start;
            const lineDecodeMs = line.decodeMs ?? DECODE_MS;
            const revealCount = Math.min(
              line.text.length,
              Math.floor((elapsed / lineDecodeMs) * line.text.length),
            );
            const decoded = revealCount >= line.text.length;

            let text = '';
            for (let c = 0; c < line.text.length; c++) {
              if (c < revealCount || isStructural(line.text[c])) {
                text += line.text[c];
              } else {
                text += GARBLE[Math.floor(Math.random() * GARBLE.length)];
              }
            }

            return (
              <div
                key={i}
                style={{ color: decoded ? line.color : 'var(--green-dim)' }}
              >
                {text}
              </div>
            );
          })}
        </div>
      )}

      {/* Phase 4: SYSTEM ONLINE */}
      {phase >= 4 && (
        <div
          style={{
            marginTop: '20px',
            fontSize: '14px',
            fontWeight: 700,
            textShadow: '0 0 8px rgba(0,255,65,0.6)',
          }}
        >
          [+] SYSTEM ONLINE. Welcome, operator.
        </div>
      )}

      {/* Skip prompt */}
      <div
        style={{
          position: 'fixed',
          bottom: '24px',
          fontSize: '11px',
          color: 'var(--green-dim)',
          opacity: 0.4,
          letterSpacing: '2px',
        }}
      >
        PRESS ANY KEY TO SKIP
      </div>
    </div>
  );
}
