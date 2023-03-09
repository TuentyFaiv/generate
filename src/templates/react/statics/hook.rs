pub static HOOK_TS: &str = r#"import { useState, useCallback } from "react";

import type {
  HookNAMEConfig,
  HookNAMEState,
  HookNAMEAction,
  HookNAMEReturn
} from "@typing/hooks/useNAME";

export function useNAME(config: HookNAMEConfig = null): HookNAMEReturn {
  const [stateNAME, setStateNAME] = useState<HookNAMEState>(null);

  const action: HookNAMEAction = useCallback(() => {
    // ...Implement
  }, []);

  return { state: stateNAME, action };
}
"#;

pub static HOOK: &str = r#"import { useState, useCallback } from "react";

export function useNAME(config = null) {
  const [stateNAME, setStateNAME] = useState(null);

  const action = useCallback(() => {
    // ...Implement
  }, []);

  return { state: stateNAME, action };
}
"#;

pub static PROPTYPES: &str = r#"export interface HookNAMEConfig {
  option: unknown;
}

// Change for interface if is an object
export type HookNAMEState = null;

export type HookNAMEAction = VoidFunction;

export interface HookNAMEReturn {
  state: HookNAMEState;
  action: HookNAMEAction;
}
"#;
