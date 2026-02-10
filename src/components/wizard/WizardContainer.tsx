import { ConnectStep } from './ConnectStep';
import { ScanStep } from './ScanStep';
import { BlankStep } from './BlankStep';
import { WriteStep } from './WriteStep';
import { VerifyStep } from './VerifyStep';
import { CompleteStep } from './CompleteStep';
import { ErrorStep } from './ErrorStep';
import { useWizard } from '../../hooks/useWizard';

export function WizardContainer() {
  const wizard = useWizard();

  const renderStep = () => {
    switch (wizard.currentStep) {
      case 'Idle':
        return <ConnectStep onConnected={wizard.detect} isLoading={false} />;
      case 'DetectingDevice':
        return <ConnectStep onConnected={wizard.detect} isLoading={true} />;
      case 'DeviceConnected':
        return (
          <ScanStep
            device={{
              model: wizard.context.model!,
              port: wizard.context.port!,
              firmware: wizard.context.firmware!,
            }}
            onScanned={wizard.scan}
            isLoading={false}
          />
        );
      case 'ScanningCard':
        return (
          <ScanStep
            device={{
              model: wizard.context.model!,
              port: wizard.context.port!,
              firmware: wizard.context.firmware!,
            }}
            onScanned={wizard.scan}
            isLoading={true}
          />
        );
      case 'CardIdentified':
        return (
          <ScanStep
            device={{
              model: wizard.context.model!,
              port: wizard.context.port!,
              firmware: wizard.context.firmware!,
            }}
            cardData={wizard.context.cardData}
            cardType={wizard.context.cardType}
            frequency={wizard.context.frequency}
            cloneable={wizard.context.cloneable}
            onScanned={() => wizard.skipToBlank(wizard.context.recommendedBlank!)}
            onReset={wizard.reset}
            isLoading={false}
          />
        );
      case 'WaitingForBlank':
        return (
          <BlankStep
            expectedBlank={wizard.context.expectedBlank}
            isLoading={true}
            onReady={() => {}}
          />
        );
      case 'BlankDetected':
        return (
          <BlankStep
            expectedBlank={wizard.context.expectedBlank}
            blankType={wizard.context.blankType}
            isLoading={false}
            onReady={wizard.write}
          />
        );
      case 'Writing':
        return (
          <WriteStep
            progress={wizard.context.writeProgress}
            currentBlock={wizard.context.currentBlock}
            totalBlocks={wizard.context.totalBlocks}
            cardType={wizard.context.cardType}
            isLoading={true}
            onComplete={() => {}}
          />
        );
      case 'Verifying':
        return (
          <VerifyStep
            isLoading={true}
            onContinue={() => {}}
          />
        );
      case 'VerificationComplete':
        return (
          <VerifyStep
            success={wizard.context.verifySuccess}
            mismatchedBlocks={wizard.context.mismatchedBlocks}
            isLoading={false}
            onContinue={wizard.finish}
            onRetryWrite={() => wizard.send({ type: 'WRITE' })}
            onReset={wizard.reset}
          />
        );
      case 'Complete':
        return (
          <CompleteStep
            cardType={wizard.context.cardType}
            cardData={wizard.context.cardData}
            timestamp={wizard.context.completionTimestamp}
            onReset={wizard.reset}
          />
        );
      case 'Error':
        return (
          <ErrorStep
            message={wizard.context.errorUserMessage}
            recoverable={wizard.context.errorRecoverable}
            recoveryAction={wizard.context.errorRecoveryAction}
            errorSource={wizard.context.errorSource}
            onRetry={wizard.reset}
            onReset={wizard.reset}
          />
        );
      default:
        return (
          <div style={{
            color: 'var(--red-bright)',
            fontSize: '13px',
            padding: '24px',
            fontFamily: 'var(--font-mono)',
          }}>
            [!!] Unknown state: {wizard.currentStep}
          </div>
        );
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
