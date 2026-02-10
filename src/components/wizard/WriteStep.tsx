import { useState, useEffect, useRef } from 'react';
import { TerminalPanel } from '../shared/TerminalPanel';
import { ProgressBar } from '../shared/ProgressBar';
import { StepIndicator, type Step } from '../shared/StepIndicator';
import { OperationLog, type LogEntry } from '../shared/OperationLog';

interface WriteStepProps {
  onComplete: () => void;
}

export function WriteStep({ onComplete }: WriteStepProps) {
  const [progress, setProgress] = useState(0);
  const [steps, setSteps] = useState<Step[]>([
    { label: 'PREPARE BLANK', status: 'pending' },
    { label: 'WRITE DATA', status: 'pending' },
    { label: 'LOCK CONFIG', status: 'pending' },
    { label: 'FINALIZE', status: 'pending' },
  ]);
  const [logLines, setLogLines] = useState<LogEntry[]>([]);
  const completedRef = useRef(false);

  useEffect(() => {
    const addLog = (entry: LogEntry) => {
      setLogLines(prev => [...prev, entry]);
    };

    const updateStep = (idx: number, status: Step['status']) => {
      setSteps(prev => prev.map((s, i) => (i === idx ? { ...s, status } : s)));
    };

    // Phase 1: PREPARE (0-25%)
    const t1 = setTimeout(() => {
      updateStep(0, 'running');
      addLog({ prefix: '=', text: 'Preparing T5577 blank...' });
    }, 200);

    const t2 = setTimeout(() => {
      setProgress(15);
      addLog({ prefix: '+', text: 'lf t55xx detect -- T5577 found' });
    }, 800);

    const t3 = setTimeout(() => {
      setProgress(25);
      updateStep(0, 'ok');
      addLog({ prefix: '+', text: 'Blank ready for write' });
    }, 1200);

    // Phase 2: WRITE (25-60%)
    const t4 = setTimeout(() => {
      updateStep(1, 'running');
      addLog({ prefix: '=', text: 'Writing EM4100 data to T5577...' });
    }, 1400);

    const t5 = setTimeout(() => {
      setProgress(40);
      addLog({ prefix: '+', text: 'lf em 410x clone --id 1A2B3C4D5E' });
    }, 2000);

    const t6 = setTimeout(() => {
      setProgress(60);
      updateStep(1, 'ok');
      addLog({ prefix: '+', text: 'Data written successfully' });
    }, 2800);

    // Phase 3: LOCK (60-80%)
    const t7 = setTimeout(() => {
      updateStep(2, 'running');
      addLog({ prefix: '=', text: 'Configuring modulation...' });
    }, 3000);

    const t8 = setTimeout(() => {
      setProgress(80);
      updateStep(2, 'ok');
      addLog({ prefix: '+', text: 'Config block set: ASK/Manchester' });
    }, 3600);

    // Phase 4: FINALIZE (80-100%)
    const t9 = setTimeout(() => {
      updateStep(3, 'running');
      addLog({ prefix: '=', text: 'Finalizing...' });
    }, 3800);

    const t10 = setTimeout(() => {
      setProgress(100);
      updateStep(3, 'ok');
      addLog({ prefix: '+', text: 'Write complete' });
    }, 4200);

    const t11 = setTimeout(() => {
      if (!completedRef.current) {
        completedRef.current = true;
        onComplete();
      }
    }, 4500);

    return () => {
      [t1, t2, t3, t4, t5, t6, t7, t8, t9, t10, t11].forEach(clearTimeout);
    };
  }, [onComplete]);

  return (
    <TerminalPanel title="WRITING">
      <div style={{ marginBottom: '16px' }}>
        <ProgressBar value={progress} width={24} />
      </div>
      <StepIndicator steps={steps} />
      <div style={{ marginTop: '16px' }}>
        <OperationLog lines={logLines} maxHeight={120} />
      </div>
    </TerminalPanel>
  );
}
