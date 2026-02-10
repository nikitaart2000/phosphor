import { useState, useEffect, useCallback } from 'react';
import { useTypewriter } from '../../hooks/useTypewriter';

interface BootSequenceProps {
  onComplete: () => void;
}

const BOOT_LINES = [
  { text: '[+] Loading card database......... [OK]  34 types', delay: 0 },
  { text: '[+] Scanning USB ports............ [OK]', delay: 150 },
  { text: '[+] System ready.', delay: 300 },
];

export function BootSequence({ onComplete }: BootSequenceProps) {
  const [phase, setPhase] = useState(1);
  const [visibleLines, setVisibleLines] = useState(0);

  const titleText = phase >= 2 ? 'PHOSPHOR v1.0' : '';
  const displayedTitle = useTypewriter(titleText, 40);

  const subText = phase >= 2 ? ':::::::::::' : '';
  const displayedSub = useTypewriter(
    displayedTitle === titleText ? subText : '',
    30
  );

  const initText = phase >= 2 ? 'INITIALIZING SYSTEM...' : '';
  const displayedInit = useTypewriter(
    displayedSub === subText ? initText : '',
    25
  );

  const skip = useCallback(() => {
    setPhase(4);
    onComplete();
  }, [onComplete]);

  // Phase transitions
  useEffect(() => {
    if (phase >= 4) return;

    const timers: ReturnType<typeof setTimeout>[] = [];

    // Phase 1 -> Phase 2 at 1.5s
    timers.push(setTimeout(() => setPhase(2), 1500));

    // Phase 2 -> Phase 3 at 2.5s
    timers.push(setTimeout(() => setPhase(3), 2500));

    // Phase 3: boot lines appear 150ms apart starting at 2.5s
    BOOT_LINES.forEach((_, i) => {
      timers.push(setTimeout(() => {
        setVisibleLines(prev => Math.max(prev, i + 1));
      }, 2500 + i * 150));
    });

    // Phase 4 at 4s
    timers.push(setTimeout(() => {
      setPhase(4);
      onComplete();
    }, 4000));

    return () => timers.forEach(t => clearTimeout(t));
  }, [onComplete, phase]);

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

  if (phase >= 4) return null;

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
      {phase >= 2 && (
        <div style={{ textAlign: 'center' }}>
          <div style={{ fontSize: '24px', fontWeight: 700, marginBottom: '8px' }}>
            {displayedTitle}
            {displayedTitle !== titleText && (
              <span style={{ opacity: 0.7 }}>_</span>
            )}
          </div>
          <div style={{ fontSize: '14px', color: 'var(--green-dim)', marginBottom: '8px' }}>
            {displayedSub}
          </div>
          <div style={{ fontSize: '14px', color: 'var(--green-mid)', marginBottom: '24px' }}>
            {displayedInit}
            {displayedInit !== initText && displayedSub === subText && (
              <span style={{ opacity: 0.7 }}>_</span>
            )}
          </div>
        </div>
      )}

      {phase >= 3 && (
        <div style={{ textAlign: 'left', fontSize: '13px', lineHeight: '1.8' }}>
          {BOOT_LINES.slice(0, visibleLines).map((line, i) => (
            <div key={i} style={{ color: 'var(--green-mid)' }}>
              {line.text}
            </div>
          ))}
        </div>
      )}

      <div
        style={{
          position: 'fixed',
          bottom: '24px',
          fontSize: '11px',
          color: 'var(--green-dim)',
          opacity: 0.5,
        }}
      >
        PRESS ANY KEY TO SKIP
      </div>
    </div>
  );
}
