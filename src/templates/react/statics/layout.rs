pub static LAYOUT_TS: &str = r#"import { useState } from "react";
import { Outlet } from "react-router-dom";

import type { NAMEState } from "@typing/layouts/NAME_LOWER";

import * as Styles from "./NAMELayout.styles";

export default function NAMEPage() {
  const [state, setState] = useState<NAMEState>(null);

  return (
    <Styles.NAME>
      <Outlet />
    </Styles.NAME>
  );
}
"#;

pub static LAYOUT: &str = r#"import { useState } from "react";
import { Outlet } from "react-router-dom";

import * as Styles from "./NAMELayout.styles";

export default function NAMEPage() {
  const [state, setState] = useState(null);

  return (
    <Styles.NAME>
      <Outlet />
    </Styles.NAME>
  );
}
"#;

pub static STYLES: &str = r#"import styled from "@emotion/styled";

import * as responsive from "./NAMELayout.styles.responsive";

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

pub static PROPTYPES: &str = r#"// Change for interface if is an object
export type NAMEState = null;
"#;
