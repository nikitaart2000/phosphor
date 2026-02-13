import { ConnectStep } from './ConnectStep';
import { ScanStep } from './ScanStep';
import { BlankStep } from './BlankStep';
import { WriteStep } from './WriteStep';
import { VerifyStep } from './VerifyStep';
import { CompleteStep } from './CompleteStep';
import { ErrorStep } from './ErrorStep';
import { FirmwareUpdateStep } from './FirmwareUpdateStep';
import { HfProcessStep } from './HfProcessStep';
import { HfDumpReadyStep } from './HfDumpReadyStep';
import { useWizard } from '../../hooks/useWizard';

export function WizardContainer() {
  const wizard = useWizard();

  const renderStep = () => {
    switch (wizard.currentStep) {
      case 'Idle':
        return <ConnectStep onConnected={wizard.detect} isLoading={false} />;
      case 'DetectingDevice':
        return <ConnectStep onConnected={wizard.detect} isLoading={true} />;
      case 'CheckingFirmware':
        return (
          <FirmwareUpdateStep
            step="CheckingFirmware"
            onUpdate={() => {}}
            onSkip={() => {}}
            onCancel={() => {}}
          />
        );
      case 'FirmwareOutdated':
        return (
          <FirmwareUpdateStep
            step="FirmwareOutdated"
            clientVersion={wizard.context.clientVersion}
            deviceFirmwareVersion={wizard.context.deviceFirmwareVersion}
            hardwareVariant={wizard.context.hardwareVariant}
            firmwarePathExists={wizard.context.firmwarePathExists}
            onUpdate={wizard.updateFirmware}
            onSkip={wizard.skipFirmware}
            onCancel={() => {}}
            onSelectVariant={wizard.selectVariant}
          />
        );
      case 'UpdatingFirmware':
        return (
          <FirmwareUpdateStep
            step="UpdatingFirmware"
            firmwareProgress={wizard.context.firmwareProgress}
            firmwareMessage={wizard.context.firmwareMessage}
            onUpdate={() => {}}
            onSkip={() => {}}
            onCancel={wizard.cancelFirmware}
          />
        );
      case 'RedetectingDevice':
        return (
          <FirmwareUpdateStep
            step="RedetectingDevice"
            onUpdate={() => {}}
            onSkip={() => {}}
            onCancel={() => {}}
          />
        );
      case 'DeviceConnected':
        return (
          <ScanStep
            device={{
              model: wizard.context.model ?? 'Unknown',
              port: wizard.context.port ?? '',
              firmware: wizard.context.firmware ?? 'Unknown',
            }}
            onScanned={wizard.scan}
            isLoading={false}
          />
        );
      case 'ScanningCard':
        return (
          <ScanStep
            device={{
              model: wizard.context.model ?? 'Unknown',
              port: wizard.context.port ?? '',
              firmware: wizard.context.firmware ?? 'Unknown',
            }}
            onScanned={wizard.scan}
            isLoading={true}
          />
        );
      case 'CardIdentified': {
        const isHf = wizard.context.frequency === 'HF';
        return (
          <ScanStep
            device={{
              model: wizard.context.model ?? 'Unknown',
              port: wizard.context.port ?? '',
              firmware: wizard.context.firmware ?? 'Unknown',
            }}
            cardData={wizard.context.cardData}
            cardType={wizard.context.cardType}
            frequency={wizard.context.frequency}
            cloneable={wizard.context.cloneable}
            skipSwapConfirm={isHf}
            onScanned={isHf
              ? () => wizard.startHfProcess()
              : () => wizard.skipToBlank(wizard.context.recommendedBlank!)
            }
            onBack={wizard.backToScan}
            onSave={async (name: string) => {
              const { saveCard } = await import('../../lib/api');
              await saveCard({
                name,
                cardType: wizard.context.cardType ?? '',
                frequency: wizard.context.frequency ?? '',
                uid: wizard.context.cardData?.uid ?? '',
                raw: wizard.context.cardData?.raw ?? '',
                decoded: JSON.stringify(wizard.context.cardData?.decoded ?? {}),
                cloneable: wizard.context.cloneable,
                recommendedBlank: wizard.context.recommendedBlank ?? '',
                createdAt: new Date().toISOString(),
              });
            }}
            isLoading={false}
          />
        );
      }
      case 'HfProcessing':
        return (
          <HfProcessStep
            cardType={wizard.context.cardType}
            phase={wizard.context.hfPhase}
            keysFound={wizard.context.hfKeysFound}
            keysTotal={wizard.context.hfKeysTotal}
            elapsed={wizard.context.hfElapsed}
            onCancel={wizard.cancelHf}
          />
        );
      case 'HfDumpReady':
        return (
          <HfDumpReadyStep
            dumpInfo={wizard.context.hfDumpInfo}
            keysFound={wizard.context.hfKeysFound}
            keysTotal={wizard.context.hfKeysTotal}
            recommendedBlank={wizard.context.recommendedBlank}
            onWriteToBlank={(blank) => wizard.skipToBlank(blank)}
            onBack={wizard.backToScan}
          />
        );
      case 'WaitingForBlank':
        return (
          <BlankStep
            expectedBlank={wizard.context.expectedBlank}
            isLoading={true}
            onReady={() => {}}
            onReset={wizard.reset}
            frequency={wizard.context.frequency}
          />
        );
      case 'BlankDetected':
        return (
          <BlankStep
            expectedBlank={wizard.context.expectedBlank}
            blankType={wizard.context.blankType}
            readyToWrite={wizard.context.readyToWrite}
            existingData={wizard.context.blankExistingData}
            isLoading={false}
            onReady={wizard.write}
            onBack={wizard.backToScan}
            frequency={wizard.context.frequency}
            onErase={async () => {
              const port = wizard.context.port;
              const blankType = wizard.context.blankType;
              if (!port || !blankType) return;
              const { wipeChip } = await import('../../lib/api');
              await wipeChip(port, blankType);
              // Re-detect blank after erase (BlankDetected -> WaitingForBlank)
              await wizard.reDetectBlank();
            }}
          />
        );
      case 'Writing':
        return (
          <WriteStep
            progress={wizard.context.writeProgress}
            currentBlock={wizard.context.currentBlock}
            totalBlocks={wizard.context.totalBlocks}
            cardType={wizard.context.cardType}
            blankType={wizard.context.blankType}
            isLoading={true}
          />
        );
      case 'Verifying':
        return (
          <VerifyStep
            isLoading={true}
            onContinue={() => {}}
            onReset={wizard.reset}
          />
        );
      case 'VerificationComplete':
        return (
          <VerifyStep
            success={wizard.context.verifySuccess}
            mismatchedBlocks={wizard.context.mismatchedBlocks}
            isLoading={false}
            onContinue={wizard.finish}
            onRetryWrite={wizard.reset}
            onReset={wizard.reset}
          />
        );
      case 'Complete':
        return (
          <CompleteStep
            cardType={wizard.context.cardType}
            cardData={wizard.context.cardData}
            timestamp={wizard.context.completionTimestamp}
            onReset={wizard.softReset}
            onDisconnect={wizard.disconnect}
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
