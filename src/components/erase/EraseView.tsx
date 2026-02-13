import { useState, useEffect, useCallback } from 'react';
import { TerminalPanel } from '../shared/TerminalPanel';
import { useSfx } from '../../hooks/useSfx';
import { detectChip, wipeChip } from '../../lib/api';
import type { DetectChipResult } from '../../lib/api';

type Phase = 'idle' | 'detecting' | 'detected' | 'erasing' | 'complete' | 'error';

interface EraseViewProps {
  port?: string;
}

const SPINNER_FRAMES = ['|', '/', '-', '\\'];

export function EraseView({ port }: EraseViewProps) {
  const sfx = useSfx();
  const [phase, setPhase] = useState<Phase>('idle');
  const [chip, setChip] = useState<DetectChipResult | null>(null);
  const [message, setMessage] = useState('');
  const [dots, setDots] = useState('');
  const [spinnerIdx, setSpinnerIdx] = useState(0);

  // Animated dots for detecting
  useEffect(() => {
    if (phase !== 'detecting') return;
    const timer = setInterval(() => {
      setDots(prev => (prev.length >= 3 ? '' : prev + '.'));
    }, 400);
    return () => clearInterval(timer);
  }, [phase]);

  // Spinner for erasing
  useEffect(() => {
    if (phase !== 'erasing') return;
    const timer = setInterval(() => {
      setSpinnerIdx(prev => (prev + 1) % SPINNER_FRAMES.length);
    }, 100);
    return () => clearInterval(timer);
  }, [phase]);

  const handleDetect = useCallback(async () => {
    if (!port || phase === 'detecting' || phase === 'erasing') return;
    setPhase('detecting');
    setChip(null);
    setMessage('');
    try {
      const result = await detectChip(port);
      setChip(result);
      setPhase('detected');
    } catch (err: unknown) {
      const msg = typeof err === 'object' && err !== null
        ? String(Object.values(err as Record<string, unknown>)[0])
        : String(err);
      setMessage(msg);
      setPhase('error');
    }
  }, [port, phase]);

  const handleErase = useCallback(async () => {
    if (!port || !chip || phase !== 'detected') return;
    setPhase('erasing');
    setMessage('');
    try {
      const result = await wipeChip(port, chip.chipType);
      if (result.success) {
        setMessage(result.message);
        setPhase('complete');
      } else {
        setMessage(result.message);
        setPhase('error');
      }
    } catch (err: unknown) {
      const msg = typeof err === 'object' && err !== null
        ? String(Object.values(err as Record<string, unknown>)[0])
        : String(err);
      setMessage(msg);
      setPhase('error');
    }
  }, [port, chip, phase]);

  const handleReset = useCallback(() => {
    setPhase('idle');
    setChip(null);
    setMessage('');
  }, []);

  const buttonStyle: React.CSSProperties = {
    background: 'var(--bg-void)',
    fontFamily: 'var(--font-mono)',
    fontSize: '13px',
    fontWeight: 600,
    padding: '8px 24px',
    cursor: 'pointer',
    textTransform: 'uppercase',
  };

  const noPort = !port;

  return (
    <TerminalPanel title="ERASE">
      <div style={{ fontSize: '13px', lineHeight: '1.8', maxWidth: '500px' }}>

        {/* No device connected */}
        {noPort && (
          <div style={{ color: 'var(--green-dim)' }}>
            [~] No device connected. Go to SCAN tab and connect a Proxmark3 first.
          </div>
        )}

        {/* Idle — ready to detect */}
        {!noPort && phase === 'idle' && (
          <>
            <div style={{ color: 'var(--green-dim)', marginBottom: '12px' }}>
              Place a card on the reader and press DETECT to identify the chip type.
            </div>
            <div style={{ color: 'var(--green-dim)', marginBottom: '16px', fontSize: '11px' }}>
              [!] This will permanently erase all data on the card.
            </div>
            <button
              onClick={() => { sfx.action(); handleDetect(); }}
              style={{
                ...buttonStyle,
                color: 'var(--green-bright)',
                border: '2px solid var(--green-bright)',
              }}
              onMouseEnter={(e) => {
                sfx.hover();
                e.currentTarget.style.background = 'var(--green-ghost)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = 'var(--bg-void)';
              }}
            >
              DETECT CHIP
            </button>
          </>
        )}

        {/* Detecting chip */}
        {phase === 'detecting' && (
          <div style={{ color: 'var(--amber)' }}>
            DETECTING CHIP{dots}
          </div>
        )}

        {/* Chip detected — show info + erase button */}
        {phase === 'detected' && chip && (
          <>
            <div style={{ color: 'var(--green-bright)', fontWeight: 700, marginBottom: '8px' }}>
              [+] CHIP DETECTED
            </div>
            <div style={{ marginBottom: '4px' }}>
              <span style={{ color: 'var(--green-dim)' }}>{'Type... '}</span>
              <span style={{ color: 'var(--green-bright)' }}>{chip.chipType}</span>
            </div>
            {chip.passwordProtected && (
              <div style={{ marginBottom: '4px' }}>
                <span style={{ color: 'var(--amber)' }}>[!] Password-protected</span>
              </div>
            )}
            <div style={{ marginBottom: '16px' }}>
              <span style={{ color: 'var(--green-dim)' }}>{chip.details}</span>
            </div>

            <div style={{
              color: 'var(--red-bright)',
              marginBottom: '16px',
              fontSize: '12px',
              fontWeight: 600,
            }}>
              [!!] WARNING: This will erase ALL data on the {chip.chipType} chip.
              {chip.passwordProtected && ' Password will be reset.'}
            </div>

            <div style={{ display: 'flex', gap: '12px' }}>
              <button
                onClick={() => { sfx.action(); handleErase(); }}
                style={{
                  ...buttonStyle,
                  color: 'var(--red-bright)',
                  border: '2px solid var(--red-bright)',
                }}
                onMouseEnter={(e) => {
                  sfx.hover();
                  e.currentTarget.style.background = 'rgba(255, 0, 51, 0.08)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'var(--bg-void)';
                }}
              >
                ERASE CHIP
              </button>
              <button
                onClick={() => { sfx.action(); handleReset(); }}
                style={{
                  ...buttonStyle,
                  color: 'var(--green-bright)',
                  border: '2px solid var(--green-bright)',
                }}
                onMouseEnter={(e) => {
                  sfx.hover();
                  e.currentTarget.style.background = 'var(--green-ghost)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'var(--bg-void)';
                }}
              >
                CANCEL
              </button>
            </div>
          </>
        )}

        {/* Erasing */}
        {phase === 'erasing' && (
          <div>
            <div style={{ color: 'var(--amber)' }}>
              [{SPINNER_FRAMES[spinnerIdx]}] Erasing {chip?.chipType ?? 'chip'}...
            </div>
            <div style={{ color: 'var(--green-dim)', marginTop: '4px', fontSize: '12px' }}>
              Do not remove the card from the reader.
            </div>
          </div>
        )}

        {/* Complete */}
        {phase === 'complete' && (
          <>
            <div style={{ color: 'var(--green-bright)', fontSize: '16px', fontWeight: 700, marginBottom: '8px' }}>
              [OK] ERASE COMPLETE
            </div>
            {message && (
              <div style={{ color: 'var(--green-dim)', marginBottom: '16px' }}>
                {message}
              </div>
            )}
            <button
              onClick={() => { sfx.action(); handleReset(); }}
              style={{
                ...buttonStyle,
                color: 'var(--green-bright)',
                border: '2px solid var(--green-bright)',
              }}
              onMouseEnter={(e) => {
                sfx.hover();
                e.currentTarget.style.background = 'var(--green-ghost)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = 'var(--bg-void)';
              }}
            >
              {'-->'} ERASE ANOTHER
            </button>
          </>
        )}

        {/* Error */}
        {phase === 'error' && (
          <>
            <div style={{ color: 'var(--red-bright)', fontSize: '16px', fontWeight: 700, marginBottom: '8px' }}>
              [!!] ERASE FAILED
            </div>
            {message && (
              <div style={{ color: 'var(--red-bright)', marginBottom: '16px', fontSize: '12px' }}>
                {message}
              </div>
            )}
            <div style={{ display: 'flex', gap: '12px' }}>
              <button
                onClick={() => { sfx.action(); handleDetect(); }}
                style={{
                  ...buttonStyle,
                  color: 'var(--amber)',
                  border: '2px solid var(--amber)',
                }}
                onMouseEnter={(e) => {
                  sfx.hover();
                  e.currentTarget.style.background = 'rgba(255, 184, 0, 0.08)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'var(--bg-void)';
                }}
              >
                RETRY
              </button>
              <button
                onClick={() => { sfx.action(); handleReset(); }}
                style={{
                  ...buttonStyle,
                  color: 'var(--green-bright)',
                  border: '2px solid var(--green-bright)',
                }}
                onMouseEnter={(e) => {
                  sfx.hover();
                  e.currentTarget.style.background = 'var(--green-ghost)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'var(--bg-void)';
                }}
              >
                RESET
              </button>
            </div>
          </>
        )}

      </div>
    </TerminalPanel>
  );
}
