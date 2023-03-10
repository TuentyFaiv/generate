pub static LAYOUT_TS: &str = r#"import { useState } from "react";

import type { NAMEProps, NAMEState } from "@typing/layouts/NAME_LOWER";

import * as Styles from "./NAME.styles";

export default function NAMEPage({ children }: NAMEProps) {
  const [state, setState] = useState<NAMEState>(null);

  return (
    <Styles.NAME>
      {children}
    </Styles.NAME>
  );
}
"#;

pub static LAYOUT: &str = r#"import { useState } from "react";

import * as Styles from "./NAME.styles";

export default function NAMEPage({ children }) {
  const [state, setState] = useState(null);

  return (
    <Styles.NAME>
      {children}
    </Styles.NAME>
  );
}
"#;

pub static STYLES: &str = r#"import styled from "@emotion/styled";

import * as responsive from "./NAME.styles.responsive";

export const NAME = styled.div`
  ${responsive.NAME_LOWER}
`;
"#;

pub static STYLES_RESPONSIVE: &str = r#"import { css } from "@emotion/react";
import { forsize } from "@mixins";

export const NAME_LOWER = css`
${forsize({ size: "desktop-mid", content: css`

` })}
`;
"#;

pub static PROPTYPES: &str = r#"import type { ReactNode } from "react";

export interface NAMEProps {
  children: ReactNode;
};

// Change for interface if is an object
export type NAMEState = null;
"#;
