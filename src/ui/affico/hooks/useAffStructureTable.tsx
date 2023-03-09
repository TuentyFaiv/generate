import { useState, useCallback } from "react";

import type {
  HookAffStructureTableConfig,
  HookAffStructureTableState,
  HookAffStructureTableAction,
  HookAffStructureTableReturn
} from "@typing/hooks/useAffStructureTable";

export function useAffStructureTable(config: HookAffStructureTableConfig = null): HookAffStructureTableReturn {
  const [stateAffStructureTable, setStateAffStructureTable] = useState<HookAffStructureTableState>(null);

  const action: HookAffStructureTableAction = useCallback(() => {
    // ...Implement
  }, []);

  return { state: stateAffStructureTable, action };
}
