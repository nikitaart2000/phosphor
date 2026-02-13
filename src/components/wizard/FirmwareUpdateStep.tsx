import { useState, useEffect } from 'react';
import { TerminalPanel } from '../shared/TerminalPanel';
import { ProgressBar } from '../shared/ProgressBar';
import { useSfx } from '../../hooks/useSfx';

interface FirmwareUpdateStepProps {
  step: 'CheckingFirmware' | 'FirmwareOutdated' | 'UpdatingFirmware' | 'RedetectingDevice';
  clientVersion?: string | null;
  deviceFirmwareVersion?: string | null;
  hardwareVariant?: string | null;
  firmwarePathExists?: boolean;
  firmwareProgress?: number;
  firmwareMessage?: string | null;
  onUpdate: () => void;
  onSkip: () => void;
  onCancel: () => void;
  onSelectVariant?: (variant: 'rdv4' | 'rdv4-bt' | 'generic') => void;
}

const VARIANT_OPTIONS: { id: 'rdv4' | 'rdv4-bt' | 'generic'; label: string }[] = [
  { id: 'rdv4', label: 'PROXMARK3 RDV4' },
  { id: 'rdv4-bt', label: 'RDV4 + BLUESHARK' },
  { id: 'generic', label: 'PM3 EASY / CLONE' },
];

const btnBase: React.CSSProperties = {
  background: 'var(--bg-void)',
  fontFamily: 'var(--font-mono)',
  fontSize: '14px',
  fontWeight: 600,
  padding: '8px 24px',
  textTransform: 'uppercase',
  cursor: 'pointer',
  border: '2px solid var(--green-bright)',
  color: 'var(--green-bright)',
};

export function FirmwareUpdateStep({
  step,
  clientVersion,
  deviceFirmwareVersion,
  hardwareVariant,
  firmwarePathExists = true,
  firmwareProgress = 0,
  firmwareMessage,
  onUpdate,
  onSkip,
  onCancel,
  onSelectVariant,
}: FirmwareUpdateStepProps) {
  const sfx = useSfx();
  const [dots, setDots] = useState('');

  const isAnimated = step === 'CheckingFirmware' || step === 'UpdatingFirmware' || step === 'RedetectingDevice';

  useEffect(() => {
    if (!isAnimated) return;
    const timer = setInterval(() => {
      setDots(prev => (prev.length >= 3 ? '' : prev + '.'));
    }, 400);
    return () => clearInterval(timer);
  }, [isAnimated]);

  if (step === 'CheckingFirmware') {
    return (
      <TerminalPanel title="FIRMWARE">
        <div style={{ color: 'var(--amber)', fontSize: '14px' }}>
          CHECKING FIRMWARE VERSION{dots}
        </div>
      </TerminalPanel>
    );
  }

  if (step === 'RedetectingDevice') {
    return (
      <TerminalPanel title="FIRMWARE">
        <div style={{ color: 'var(--amber)', fontSize: '14px' }}>
          RE-DETECTING DEVICE{dots}
        </div>
        <div style={{ color: 'var(--green-dim)', fontSize: '12px', marginTop: '8px' }}>
          Port may have changed after firmware update.
        </div>
      </TerminalPanel>
    );
  }

  if (step === 'UpdatingFirmware') {
    return (
      <TerminalPanel title="FIRMWARE UPDATE">
        <div style={{ fontSize: '13px', lineHeight: '1.8' }}>
          <div style={{ color: 'var(--amber)', marginBottom: '8px' }}>
            FLASHING FIRMWARE{dots}
          </div>

          <div style={{ marginBottom: '12px' }}>
            <ProgressBar value={firmwareProgress} width={24} />
          </div>

          {firmwareMessage && (
            <div style={{ color: 'var(--green-dim)', fontSize: '12px', marginBottom: '12px' }}>
              {firmwareMessage}
            </div>
          )}

          <div style={{ color: 'var(--red-bright)', fontSize: '12px', marginBottom: '16px' }}>
            [!!] DO NOT DISCONNECT THE DEVICE
          </div>

          <button
            onClick={() => { sfx.action(); onCancel(); }}
            style={{
              ...btnBase,
              border: '2px solid var(--red-bright)',
              color: 'var(--red-bright)',
            }}
            onMouseEnter={(e) => {
              sfx.hover();
              e.currentTarget.style.background = 'rgba(255, 0, 51, 0.1)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = 'var(--bg-void)';
            }}
          >
            CANCEL FLASH
          </button>
        </div>
      </TerminalPanel>
    );
  }

  // step === 'FirmwareOutdated' — variant picker when hardware is unknown
  if (hardwareVariant === 'unknown' || !hardwareVariant) {
    return (
      <TerminalPanel title="SELECT HARDWARE">
        <div style={{ fontSize: '13px', lineHeight: '1.8' }}>
          <div style={{ color: 'var(--amber)', marginBottom: '12px' }}>
            [!] Firmware mismatch — hardware variant could not be detected automatically.
          </div>
          <div style={{ color: 'var(--green-dim)', fontSize: '12px', marginBottom: '16px' }}>
            Select your Proxmark3 model to flash the correct firmware:
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', marginBottom: '16px' }}>
            {VARIANT_OPTIONS.map((opt) => (
              <button
                key={opt.id}
                onClick={() => { sfx.action(); onSelectVariant?.(opt.id); }}
                style={{
                  ...btnBase,
                  padding: '10px 16px',
                  textAlign: 'left',
                }}
                onMouseEnter={(e) => {
                  sfx.hover();
                  e.currentTarget.style.background = 'var(--green-ghost)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'var(--bg-void)';
                }}
              >
                {opt.label}
              </button>
            ))}
          </div>

          <button
            onClick={() => { sfx.action(); onSkip(); }}
            style={{
              ...btnBase,
              border: '2px solid var(--green-dim)',
              color: 'var(--green-dim)',
              opacity: 0.7,
            }}
            onMouseEnter={(e) => {
              sfx.hover();
              e.currentTarget.style.opacity = '1';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.opacity = '0.7';
            }}
          >
            SKIP
          </button>
        </div>
      </TerminalPanel>
    );
  }

  // step === 'FirmwareOutdated' — variant known, show update/skip
  return (
    <TerminalPanel title="FIRMWARE MISMATCH">
      <div style={{ fontSize: '13px', lineHeight: '1.8' }}>
        <div style={{ color: 'var(--amber)', marginBottom: '12px' }}>
          [!] Firmware version mismatch detected
        </div>

        <div style={{ marginBottom: '4px' }}>
          <span style={{ color: 'var(--green-dim)' }}>Client:  </span>
          <span style={{ color: 'var(--green-bright)' }}>{clientVersion ?? 'unknown'}</span>
        </div>
        <div style={{ marginBottom: '4px' }}>
          <span style={{ color: 'var(--green-dim)' }}>Device:  </span>
          <span style={{ color: 'var(--red-bright)' }}>{deviceFirmwareVersion ?? 'unknown'}</span>
        </div>
        <div style={{ marginBottom: '12px' }}>
          <span style={{ color: 'var(--green-dim)' }}>HW:      </span>
          <span style={{ color: 'var(--green-bright)' }}>{hardwareVariant}</span>
        </div>

        {firmwarePathExists ? (
          <>
            <div style={{ color: 'var(--green-dim)', fontSize: '12px', marginBottom: '16px' }}>
              Updating firmware ensures all commands work correctly.
              This will flash fullimage.elf only (safe, non-bricking).
            </div>

            <div style={{ display: 'flex', gap: '12px' }}>
              <button
                onClick={() => { sfx.action(); onUpdate(); }}
                style={btnBase}
                onMouseEnter={(e) => {
                  sfx.hover();
                  e.currentTarget.style.background = 'var(--green-ghost)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'var(--bg-void)';
                }}
              >
                UPDATE FIRMWARE
              </button>
              <button
                onClick={() => { sfx.action(); onSkip(); }}
                style={{
                  ...btnBase,
                  border: '2px solid var(--green-dim)',
                  color: 'var(--green-dim)',
                  opacity: 0.7,
                }}
                onMouseEnter={(e) => {
                  sfx.hover();
                  e.currentTarget.style.opacity = '1';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.opacity = '0.7';
                }}
              >
                SKIP
              </button>
            </div>
          </>
        ) : (
          <>
            <div style={{ color: 'var(--amber)', fontSize: '12px', marginBottom: '12px' }}>
              [!] No bundled firmware for this hardware variant ({hardwareVariant}).
              <br />
              Flash manually via CLI: proxmark3 --flash --image fullimage.elf
            </div>

            <button
              onClick={() => { sfx.action(); onSkip(); }}
              style={btnBase}
              onMouseEnter={(e) => {
                sfx.hover();
                e.currentTarget.style.background = 'var(--green-ghost)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = 'var(--bg-void)';
              }}
            >
              CONTINUE
            </button>
          </>
        )}
      </div>
    </TerminalPanel>
  );
}
