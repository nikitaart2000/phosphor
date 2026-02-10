import { useMemo } from 'react';
import { TerminalPanel } from '../shared/TerminalPanel';
import { ProgressBar } from '../shared/ProgressBar';
import { StepIndicator, type Step } from '../shared/StepIndicator';
import type { CardType } from '../../machines/types';

interface WriteStepProps {
  onComplete: () => void;
  isLoading?: boolean;
  progress?: number;
  currentBlock?: number | null;
  totalBlocks?: number | null;
  cardType?: CardType | null;
}

// Derive step statuses from the progress percentage
function getPhaseSteps(progress: number): Step[] {
  // DETECT (0-10%), SAFETY CHECK (10-25%), WIPE (25-50%), CLONE (50-75%), FINALIZE (75-100%)
  const phases: { label: string; start: number; end: number }[] = [
    { label: 'DETECT BLANK', start: 0, end: 10 },
    { label: 'SAFETY CHECK', start: 10, end: 25 },
    { label: 'WIPE', start: 25, end: 50 },
    { label: 'CLONE DATA', start: 50, end: 75 },
    { label: 'FINALIZE', start: 75, end: 100 },
  ];

  return phases.map(({ label, start, end }): Step => {
    if (progress >= end) {
      return { label, status: 'ok' };
    } else if (progress >= start) {
      return { label, status: 'running' };
    } else {
      return { label, status: 'pending' };
    }
  });
}

export function WriteStep({
  onComplete: _onComplete,
  isLoading,
  progress = 0,
  currentBlock,
  totalBlocks,
  cardType,
}: WriteStepProps) {
  const steps = useMemo(() => getPhaseSteps(progress), [progress]);

  const blockInfo = currentBlock !== null && currentBlock !== undefined && totalBlocks
    ? `Block ${currentBlock}/${totalBlocks}`
    : null;

  return (
    <TerminalPanel title="WRITING">
      <div style={{ fontSize: '13px', lineHeight: '1.8' }}>
        {cardType && (
          <div style={{ color: 'var(--green-dim)', marginBottom: '8px' }}>
            Cloning {cardType} to blank...
          </div>
        )}

        <div style={{ marginBottom: '16px' }}>
          <ProgressBar value={progress} width={24} />
          {blockInfo && (
            <span style={{ color: 'var(--green-dim)', fontSize: '12px', marginLeft: '12px' }}>
              {blockInfo}
            </span>
          )}
        </div>

        <StepIndicator steps={steps} />

        {isLoading && progress >= 100 && (
          <div style={{ color: 'var(--green-bright)', marginTop: '12px', fontWeight: 600 }}>
            [+] Write complete -- verifying...
          </div>
        )}
      </div>
    </TerminalPanel>
  );
}
