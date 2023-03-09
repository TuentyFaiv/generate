import { useState, useCallback } from "react";

export function useCalendar(config = null) {
  const [stateCalendar, setStateCalendar] = useState(null);

  const action = useCallback(() => {
    // ...Implement
  }, []);

  return { state: stateCalendar, action };
}
