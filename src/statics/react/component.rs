pub static PROPTYPES: &str = r#"import type { ReactNode } from "react";

export interface Props {
  children: ReactNode;
}
"#;

pub static STYLES: &str = r#"import styled from "@emotion/styled";

import * as responsive from "./NAME_PASCAL.styles.responsive";

export const NAME_PASCAL = styled.div`
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

pub static COMPONENT: &str = r#"import * as Styles from "./NAME_PASCAL.styles";

export default function NAME_PASCAL({ children }) {
  return (
    <Styles.NAME_PASCAL>
      {children}
    </Styles.NAME_PASCAL>
  );
}
"#;

pub static COMPONENT_TS: &str = r#"import type { Props } from "./NAME_PASCAL.proptypes";

import * as Styles from "./NAME_PASCAL.styles";

export default function NAME_PASCAL({ children }: Props) {
  return (
    <Styles.NAME_PASCAL>
      {children}
    </Styles.NAME_PASCAL>
  );
}
"#;