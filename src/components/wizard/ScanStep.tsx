import { useState, useEffect, useRef } from 'react';
import { TerminalPanel } from '../shared/TerminalPanel';
import { useSfx } from '../../hooks/useSfx';
import type { CardData, CardType, Frequency } from '../../machines/types';

interface ScanStepProps {
  device: { model: string; port: string; firmware: string };
  onScanned: () => void;
  isLoading?: boolean;
  cardData?: CardData | null;
  cardType?: CardType | null;
  frequency?: Frequency | null;
  cloneable?: boolean;
}

const SPINNER_FRAMES = ['|', '/', '-', '\\'];

export function ScanStep({
  device,
  onScanned,
  isLoading,
  cardData,
  cardType,
  frequency,
  cloneable,
}: ScanStepProps) {
  const sfx = useSfx();
  const [spinnerIdx, setSpinnerIdx] = useState(0);
  const [pulseOn, setPulseOn] = useState(false);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // Border pulse animation when idle (not scanning, no result)
  useEffect(() => {
    if (isLoading || cardData) return;
    const timer = setInterval(() => setPulseOn(p => !p), 800);
    return () => clearInterval(timer);
  }, [isLoading, cardData]);

  // Spinner for scanning state
  useEffect(() => {
    if (!isLoading) return;
    intervalRef.current = setInterval(() => {
      setSpinnerIdx(prev => (prev + 1) % SPINNER_FRAMES.length);
    }, 100);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [isLoading]);

  // Card has been identified -- show results
  if (cardData && cardType) {
    const freqLabel = frequency === 'LF' ? '125 kHz (LF)' : frequency === 'HF' ? '13.56 MHz (HF)' : 'Unknown';
    const decodedEntries = cardData.decoded ? Object.entries(cardData.decoded) : [];

    return (
      <TerminalPanel title="SCAN RESULT">
        <div style={{ fontSize: '13px', lineHeight: '1.8' }}>
          <div style={{ color: 'var(--green-bright)' }}>
            [+] Card detected
          </div>
          <div style={{ color: 'var(--green-dim)', marginTop: '8px' }}>
            TYPE   : {cardType}
          </div>
          <div style={{ color: 'var(--green-dim)' }}>
            UID    : {cardData.uid}
          </div>
          <div style={{ color: 'var(--green-dim)' }}>
            FREQ   : {freqLabel}
          </div>
          {decodedEntries.map(([key, value]) => (
            <div key={key} style={{ color: 'var(--green-dim)' }}>
              {key.toUpperCase().padEnd(7)}: {value}
            </div>
          ))}

          {cloneable === false && (
            <div style={{ color: 'var(--amber)', marginTop: '12px', fontWeight: 600 }}>
              [!!] This card type cannot be cloned
            </div>
          )}

          {cloneable !== false && (
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
          )}
        </div>
      </TerminalPanel>
    );
  }

  // Scanning or waiting to scan
  return (
    <div style={{ textAlign: 'center' }}>
      {/* Device info banner */}
      <div style={{
        fontSize: '11px',
        color: 'var(--green-dim)',
        marginBottom: '16px',
        lineHeight: '1.6',
      }}>
        <span>{device.model}</span>
        <span style={{ margin: '0 8px' }}>|</span>
        <span>{device.port}</span>
        <span style={{ margin: '0 8px' }}>|</span>
        <span>{device.firmware}</span>
      </div>

      <button
        onClick={() => { if (!isLoading) sfx.action(); if (!isLoading) onScanned(); }}
        disabled={isLoading}
        style={{
          background: isLoading ? 'var(--bg-surface)' : 'var(--bg-void)',
          color: 'var(--green-bright)',
          border: `2px solid ${pulseOn && !isLoading ? 'var(--green-mid)' : 'var(--green-bright)'}`,
          fontFamily: 'var(--font-mono)',
          fontSize: '20px',
          fontWeight: 700,
          padding: '16px 48px',
          cursor: isLoading ? 'default' : 'pointer',
          transition: 'border-color 0.3s, background 0.2s',
        }}
        onMouseEnter={(e) => {
          if (!isLoading) { sfx.hover(); e.currentTarget.style.background = 'var(--green-ghost)'; }
        }}
        onMouseLeave={(e) => {
          if (!isLoading) e.currentTarget.style.background = 'var(--bg-void)';
        }}
      >
        {isLoading
          ? `[ SCANNING ${SPINNER_FRAMES[spinnerIdx]} ]`
          : '[ SCAN ]'}
      </button>
      <div style={{
        marginTop: '12px',
        fontSize: '12px',
        color: 'var(--green-dim)',
      }}>
        {isLoading ? 'Hold card on reader...' : 'Place card on reader and press SCAN'}
      </div>
    </div>
  );
}
