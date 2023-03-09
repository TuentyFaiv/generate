import { useState, useCallback } from "react";

import type {
  HookDatasConfig,
  HookDatasState,
  HookDatasAction,
  HookDatasReturn
} from "@typing/hooks/useDatas";

export function useDatas(config: HookDatasConfig = null): HookDatasReturn {
  const [stateDatas, setStateDatas] = useState<HookDatasState>(null);

  const action: HookDatasAction = useCallback(() => {
    // ...Implement
  }, []);

  return { state: stateDatas, action };
}
