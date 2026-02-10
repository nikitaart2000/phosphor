import { ConnectStep } from './ConnectStep';
import { ScanStep } from './ScanStep';
import { BlankStep } from './BlankStep';
import { WriteStep } from './WriteStep';
import { VerifyStep } from './VerifyStep';
import { CompleteStep } from './CompleteStep';

export type WizardStep = 'connect' | 'scan' | 'blank' | 'write' | 'verify' | 'complete';

interface WizardContainerProps {
  currentStep: WizardStep;
  onStepChange: (step: WizardStep) => void;
}

export function WizardContainer({ currentStep, onStepChange }: WizardContainerProps) {
  const renderStep = () => {
    switch (currentStep) {
      case 'connect':
        return <ConnectStep onConnected={() => onStepChange('scan')} />;
      case 'scan':
        return <ScanStep onScanned={() => onStepChange('blank')} />;
      case 'blank':
        return <BlankStep onReady={() => onStepChange('write')} />;
      case 'write':
        return <WriteStep onComplete={() => onStepChange('verify')} />;
      case 'verify':
        return <VerifyStep onContinue={() => onStepChange('complete')} />;
      case 'complete':
        return <CompleteStep onReset={() => onStepChange('connect')} />;
    }
  };

  return (
    <div
      style={{
        flex: 1,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '24px',
        position: 'relative',
        zIndex: 5,
      }}
    >
      {renderStep()}
    </div>
  );
}
