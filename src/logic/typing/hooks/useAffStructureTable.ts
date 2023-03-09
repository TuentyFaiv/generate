export interface HookNAMEConfig {
  option: unknown;
}

// Change for interface if is an object
export type HookNAMEState = null;

export type HookNAMEAction = VoidFunction;

export interface HookNAMEReturn {
  state: HookNAMEState;
  action: HookNAMEAction;
}
