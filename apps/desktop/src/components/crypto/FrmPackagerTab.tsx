import React from 'react';
import { ExtrheadUnpackReport, FrmPackReport, PackFrmRequest } from '@/models';
import { UnpackExtrheadParams } from '@/viewmodels/useFrmPackager';
import { FrmPackCard } from './FrmPackCard';
import { ExtrheadUnpackCard } from './ExtrheadUnpackCard';

interface FrmPackagerTabProps {
  packLoading: boolean;
  packError: string | null;
  packResult: FrmPackReport | null;
  onPack: (request: PackFrmRequest) => Promise<FrmPackReport | null>;
  onInspectOutput: (outPath: string) => void;
  unpackLoading: boolean;
  unpackError: string | null;
  unpackResult: ExtrheadUnpackReport | null;
  onUnpack: (params: UnpackExtrheadParams) => Promise<ExtrheadUnpackReport | null>;
}

export const FrmPackagerTab: React.FC<FrmPackagerTabProps> = ({
  packLoading,
  packError,
  packResult,
  onPack,
  onInspectOutput,
  unpackLoading,
  unpackError,
  unpackResult,
  onUnpack,
}) => {
  return (
    <div className="space-y-6 font-mono">
      <FrmPackCard
        packLoading={packLoading}
        packError={packError}
        packResult={packResult}
        onPack={onPack}
        onInspectOutput={onInspectOutput}
      />
      <ExtrheadUnpackCard
        unpackLoading={unpackLoading}
        unpackError={unpackError}
        unpackResult={unpackResult}
        onUnpack={onUnpack}
      />
    </div>
  );
};
