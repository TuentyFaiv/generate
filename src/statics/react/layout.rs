pub static LAYOUT_TS: &str = r#"import { useState } from "react";
import { Outlet } from "react-router-dom";

import type { NAME_CAPITALLayoutState } from "@typing/layouts/NAME_LOWER";

import { Layout } from "@NAME_LOWER/styles";

export default function NAME_CAPITALLayout() {
  const [state, setState] = useState<NAME_CAPITALLayoutState>(null);

  return (
    <Layout.NAME_CAPITAL>
      <Outlet />
    </Layout.NAME_CAPITAL>
  );
}
"#;

pub static LAYOUT: &str = r#"import { useState } from "react";
import { Outlet } from "react-router-dom";

import { Layout } from "@NAME_LOWER/styles";

export default function NAME_CAPITALLayout() {
  const [state, setState] = useState(null);

  return (
    <Layout.NAME_CAPITAL>
      <Outlet />
    </Layout.NAME_CAPITAL>
  );
}
"#;

pub static STYLES: &str = r#"import styled from "@emotion/styled";

import * as responsive from "./NAMELayout.styles.responsive";

export const NAME_CAPITAL = styled.div`
  ${responsive.NAME_LOWER}
`;
"#;

pub static STYLES_RESPONSIVE: &str = r#"import { css } from "@emotion/react";
import { forsize } from "@mixins";

export const NAME_LOWER = forsize({
  "desktop-mid": css``,
  desktop: css``,
});
"#;

pub static PROPTYPES: &str = r#"// Change for interface if is an object
export type NAME_CAPITALLayoutState = null;
"#;
