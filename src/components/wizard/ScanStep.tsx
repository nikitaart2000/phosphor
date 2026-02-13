import { useState, useEffect, useRef } from 'react';
import { TerminalPanel } from '../shared/TerminalPanel';
import { useSfx } from '../../hooks/useSfx';
import type { CardData, CardType, Frequency } from '../../machines/types';

interface ScanStepProps {
  device: { model: string; port: string; firmware: string };
  onScanned: () => void;
  onBack?: () => void;
  onSave?: (name: string) => Promise<void>;

  isLoading?: boolean;
  cardData?: CardData | null;
  cardType?: CardType | null;
  frequency?: Frequency | null;
  cloneable?: boolean;
  /** When true, WRITE button skips swap card dialog (HF cards need source on reader for autopwn) */
  skipSwapConfirm?: boolean;
}

const SPINNER_FRAMES = ['|', '/', '-', '\\'];

export function ScanStep({
  device,
  onScanned,
  onBack,
  onSave,

  isLoading,
  cardData,
  cardType,
  frequency,
  cloneable,
  skipSwapConfirm,
}: ScanStepProps) {
  const sfx = useSfx();
  const [spinnerIdx, setSpinnerIdx] = useState(0);
  const [pulseOn, setPulseOn] = useState(false);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const [showSaveInput, setShowSaveInput] = useState(false);
  const [saveName, setSaveName] = useState('');
  const [saveStatus, setSaveStatus] = useState<'idle' | 'saving' | 'saved' | 'error'>('idle');
  const [savedDisplayName, setSavedDisplayName] = useState('');
  const [showWriteConfirm, setShowWriteConfirm] = useState(false);

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
    // Filter out fields already shown in the header (type, uid) to avoid duplication
    const decodedEntries = cardData.decoded
      ? Object.entries(cardData.decoded).filter(([key]) => key !== 'type' && key !== 'uid')
      : [];

    const btnBase: React.CSSProperties = {
      background: 'var(--bg-void)',
      fontFamily: 'var(--font-mono)',
      fontSize: '13px',
      fontWeight: 600,
      padding: '6px 20px',
      cursor: 'pointer',
    };

    const handleSave = async () => {
      if (!saveName.trim() || !onSave) return;
      setSaveStatus('saving');
      try {
        await onSave(saveName.trim());
        setSavedDisplayName(saveName.trim());
        setSaveStatus('saved');
        setShowSaveInput(false);
        setSaveName('');
        setTimeout(() => setSaveStatus('idle'), 3000);
      } catch {
        setSaveStatus('error');
        setTimeout(() => setSaveStatus('idle'), 3000);
      }
    };

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

          {/* Action buttons */}
          <div style={{ marginTop: '16px', display: 'flex', gap: '12px', flexWrap: 'wrap' }}>
            {/* BACK button */}
            {onBack && (
              <button
                onClick={() => { sfx.action(); onBack(); }}
                style={{
                  ...btnBase,
                  color: 'var(--green-dim)',
                  border: '2px solid var(--green-dim)',
                }}
                onMouseEnter={(e) => {
                  sfx.hover();
                  e.currentTarget.style.background = 'var(--green-ghost)';
                  e.currentTarget.style.color = 'var(--green-bright)';
                  e.currentTarget.style.borderColor = 'var(--green-bright)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'var(--bg-void)';
                  e.currentTarget.style.color = 'var(--green-dim)';
                  e.currentTarget.style.borderColor = 'var(--green-dim)';
                }}
              >
                {'<--'} BACK
              </button>
            )}

            {/* SAVE button */}
            {onSave && (
              <button
                onClick={() => {
                  sfx.action();
                  setShowSaveInput(true);
                  setSaveStatus('idle');
                }}
                disabled={showSaveInput}
                style={{
                  ...btnBase,
                  color: 'var(--amber)',
                  border: '2px solid var(--amber)',
                  opacity: showSaveInput ? 0.5 : 1,
                  cursor: showSaveInput ? 'default' : 'pointer',
                }}
                onMouseEnter={(e) => {
                  if (!showSaveInput) {
                    sfx.hover();
                    e.currentTarget.style.background = 'rgba(255, 184, 0, 0.08)';
                  }
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'var(--bg-void)';
                }}
              >
                [SAVE]
              </button>
            )}

            {/* WRITE button (only for cloneable) */}
            {cloneable !== false && (
              <button
                onClick={() => { sfx.action(); skipSwapConfirm ? onScanned() : setShowWriteConfirm(true); }}
                style={{
                  ...btnBase,
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
                {'-->'} WRITE
              </button>
            )}
          </div>

          {/* Inline save name input */}
          {showSaveInput && (
            <div style={{ marginTop: '12px', display: 'flex', alignItems: 'center', gap: '8px' }}>
              <span style={{ color: 'var(--green-dim)' }}>NAME:</span>
              <input
                autoFocus
                value={saveName}
                onChange={(e) => setSaveName(e.target.value)}
                onKeyDown={(e) => { if (e.key === 'Enter') handleSave(); if (e.key === 'Escape') { setShowSaveInput(false); setSaveName(''); } }}
                style={{
                  background: 'var(--bg-void)',
                  color: 'var(--green-bright)',
                  border: '1px solid var(--green-dim)',
                  fontFamily: 'var(--font-mono)',
                  fontSize: '13px',
                  padding: '4px 8px',
                  flex: 1,
                  maxWidth: '240px',
                  outline: 'none',
                }}
                placeholder="card name..."
              />
              <button
                onClick={handleSave}
                disabled={!saveName.trim() || saveStatus === 'saving'}
                style={{
                  ...btnBase,
                  color: 'var(--green-bright)',
                  border: '1px solid var(--green-bright)',
                  padding: '4px 12px',
                  opacity: !saveName.trim() || saveStatus === 'saving' ? 0.5 : 1,
                  cursor: !saveName.trim() || saveStatus === 'saving' ? 'default' : 'pointer',
                }}
              >
                OK
              </button>
              <button
                onClick={() => { setShowSaveInput(false); setSaveName(''); }}
                style={{
                  ...btnBase,
                  color: 'var(--red-bright, #f33)',
                  border: '1px solid var(--red-bright, #f33)',
                  padding: '4px 12px',
                }}
              >
                X
              </button>
            </div>
          )}

          {/* Save status messages */}
          {saveStatus === 'saved' && (
            <div style={{ marginTop: '8px', color: 'var(--green-bright)' }}>
              [+] Saved as "{savedDisplayName}"
            </div>
          )}
          {saveStatus === 'error' && (
            <div style={{ marginTop: '8px', color: 'var(--red-bright, #f33)' }}>
              [!!] Save failed
            </div>
          )}
        </div>

        {/* Write confirmation overlay */}
        {showWriteConfirm && (
          <div style={{
            position: 'fixed',
            inset: 0,
            background: 'rgba(0, 0, 0, 0.85)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 100,
          }}>
            <div style={{
              border: '1px solid var(--green-bright)',
              boxShadow: '0 0 12px rgba(0, 255, 65, 0.25), inset 0 0 6px rgba(0, 255, 65, 0.05)',
              background: 'var(--bg-void)',
              padding: '24px 32px',
              maxWidth: '380px',
              fontFamily: 'var(--font-mono)',
              fontSize: '13px',
              lineHeight: '1.8',
            }}>
              <div style={{ color: 'var(--amber)', fontWeight: 700, marginBottom: '12px' }}>
                [!] SWAP CARDS
              </div>
              <div style={{ color: 'var(--green-dim)', marginBottom: '8px' }}>
                1. Remove the scanned card from the reader
              </div>
              <div style={{ color: 'var(--green-dim)', marginBottom: '16px' }}>
                2. Place the blank card you want to write to
              </div>
              <div style={{ display: 'flex', gap: '12px' }}>
                <button
                  onClick={() => { sfx.action(); setShowWriteConfirm(false); onScanned(); }}
                  style={{
                    ...btnBase,
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
                  READY
                </button>
                <button
                  onClick={() => { sfx.action(); setShowWriteConfirm(false); }}
                  style={{
                    ...btnBase,
                    color: 'var(--green-dim)',
                    border: '2px solid var(--green-dim)',
                  }}
                  onMouseEnter={(e) => {
                    sfx.hover();
                    e.currentTarget.style.background = 'var(--green-ghost)';
                    e.currentTarget.style.color = 'var(--green-bright)';
                    e.currentTarget.style.borderColor = 'var(--green-bright)';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.background = 'var(--bg-void)';
                    e.currentTarget.style.color = 'var(--green-dim)';
                    e.currentTarget.style.borderColor = 'var(--green-dim)';
                  }}
                >
                  CANCEL
                </button>
              </div>
            </div>
          </div>
        )}
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
      <div style={{
        marginTop: '24px',
        fontSize: '11px',
        color: 'var(--green-dim)',
        opacity: 0.35,
        lineHeight: '1.6',
        maxWidth: '320px',
        margin: '24px auto 0',
      }}>
        {isLoading
          ? 'Checking both LF and HF frequencies'
          : 'Your reader has two antenna spots: coil side = LF (125 kHz), opposite side = HF (13.56 MHz). Try both if not detected.'}
      </div>
    </div>
  );
}
