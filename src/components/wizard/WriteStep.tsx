import { useMemo } from 'react';
import { TerminalPanel } from '../shared/TerminalPanel';
import { ProgressBar } from '../shared/ProgressBar';
import { StepIndicator, type Step } from '../shared/StepIndicator';
import type { BlankType, CardType } from '../../machines/types';

interface WriteStepProps {
  isLoading?: boolean;
  progress?: number;
  currentBlock?: number | null;
  totalBlocks?: number | null;
  cardType?: CardType | null;
  blankType?: BlankType | null;
}

// Phase definitions matching the Rust write flow step counts.
// T5577: 6 steps (detect, password check, wipe, verify wipe, clone, finalize)
// EM4305: 5 steps (detect, wipe, verify wipe, clone, finalize)
const T5577_PHASES: { label: string; start: number; end: number }[] = [
  { label: 'DETECT BLANK', start: 0, end: 10 },
  { label: 'PASSWORD CHECK', start: 10, end: 20 },
  { label: 'WIPE', start: 20, end: 35 },
  { label: 'VERIFY WIPE', start: 35, end: 50 },
  { label: 'CLONE DATA', start: 50, end: 75 },
  { label: 'FINALIZE', start: 75, end: 100 },
];

const EM4305_PHASES: { label: string; start: number; end: number }[] = [
  { label: 'DETECT BLANK', start: 0, end: 10 },
  { label: 'WIPE', start: 10, end: 30 },
  { label: 'VERIFY WIPE', start: 30, end: 50 },
  { label: 'CLONE DATA', start: 50, end: 75 },
  { label: 'FINALIZE', start: 75, end: 100 },
];

function getPhaseSteps(progress: number, blankType?: BlankType | null): Step[] {
  const phases = blankType === 'EM4305' ? EM4305_PHASES : T5577_PHASES;

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
  isLoading,
  progress = 0,
  currentBlock,
  totalBlocks,
  cardType,
  blankType,
}: WriteStepProps) {
  const steps = useMemo(() => getPhaseSteps(progress, blankType), [progress, blankType]);

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
