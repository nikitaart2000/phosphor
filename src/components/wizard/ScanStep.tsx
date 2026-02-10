import { useState, useEffect, useRef } from 'react';
import { TerminalPanel } from '../shared/TerminalPanel';
import { useSfx } from '../../hooks/useSfx';

interface ScanStepProps {
  onScanned: () => void;
}

const SPINNER_FRAMES = ['|', '/', '-', '\\'];

export function ScanStep({ onScanned }: ScanStepProps) {
  const sfx = useSfx();
  const [scanning, setScanning] = useState(false);
  const [scanned, setScanned] = useState(false);
  const [spinnerIdx, setSpinnerIdx] = useState(0);
  const [pulseOn, setPulseOn] = useState(false);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // Border pulse animation on hover
  useEffect(() => {
    if (scanning) return;
    const timer = setInterval(() => setPulseOn(p => !p), 800);
    return () => clearInterval(timer);
  }, [scanning]);

  // Spinner for scanning state
  useEffect(() => {
    if (!scanning) return;
    intervalRef.current = setInterval(() => {
      setSpinnerIdx(prev => (prev + 1) % SPINNER_FRAMES.length);
    }, 100);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [scanning]);

  const handleScan = () => {
    setScanning(true);
    // Simulate scan for 2.5s
    setTimeout(() => {
      setScanning(false);
      setScanned(true);
    }, 2500);
  };

  if (scanned) {
    return (
      <TerminalPanel title="SCAN RESULT">
        <div style={{ fontSize: '13px', lineHeight: '1.8' }}>
          <div style={{ color: 'var(--green-bright)' }}>
            [+] Card detected
          </div>
          <div style={{ color: 'var(--green-dim)', marginTop: '8px' }}>
            TYPE   : EM4100
          </div>
          <div style={{ color: 'var(--green-dim)' }}>
            UID    : 0x1A2B3C4D5E
          </div>
          <div style={{ color: 'var(--green-dim)' }}>
            FREQ   : 125 kHz (LF)
          </div>
          <div style={{ color: 'var(--green-dim)' }}>
            BITS   : 40
          </div>
          <div style={{ marginTop: '16px' }}>
            <button
              onClick={() => { sfx.action(); onScanned(); }}
              style={{
                background: 'var(--bg-void)',
                color: 'var(--green-bright)',
                border: '2px solid var(--green-bright)',
                fontFamily: 'var(--font-mono)',
                fontSize: '13px',
                fontWeight: 600,
                padding: '6px 20px',
                cursor: 'pointer',
              }}
              onMouseEnter={(e) => {
                sfx.hover();
                e.currentTarget.style.background = 'var(--green-ghost)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = 'var(--bg-void)';
              }}
            >
              {'-->'} CONTINUE TO WRITE
            </button>
          </div>
        </div>
      </TerminalPanel>
    );
  }

  return (
    <div style={{ textAlign: 'center' }}>
      <button
        onClick={() => { if (!scanning) sfx.action(); handleScan(); }}
        disabled={scanning}
        style={{
          background: scanning ? 'var(--bg-surface)' : 'var(--bg-void)',
          color: 'var(--green-bright)',
          border: `2px solid ${pulseOn && !scanning ? 'var(--green-mid)' : 'var(--green-bright)'}`,
          fontFamily: 'var(--font-mono)',
          fontSize: '20px',
          fontWeight: 700,
          padding: '16px 48px',
          cursor: scanning ? 'default' : 'pointer',
          transition: 'border-color 0.3s, background 0.2s',
        }}
        onMouseEnter={(e) => {
          if (!scanning) { sfx.hover(); e.currentTarget.style.background = 'var(--green-ghost)'; }
        }}
        onMouseLeave={(e) => {
          if (!scanning) e.currentTarget.style.background = 'var(--bg-void)';
        }}
      >
        {scanning
          ? `[ SCANNING ${SPINNER_FRAMES[spinnerIdx]} ]`
          : '[ SCAN ]'}
      </button>
      <div style={{
        marginTop: '12px',
        fontSize: '12px',
        color: 'var(--green-dim)',
      }}>
        {scanning ? 'Hold card on reader...' : 'Place card on reader and press SCAN'}
      </div>
    </div>
  );
}
