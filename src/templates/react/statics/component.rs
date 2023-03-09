pub static PROPTYPES: &str = r#"import type { ReactNode } from "react";

export interface Props {
  children: ReactNode;
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

pub static COMPONENT: &str = r#"
import * as Styles from "./NAME.styles";

export default function NAME({ children }) {
  return (
    <Styles.NAME>
      {children}
    </Styles.NAME>
  )
}
"#;

pub static COMPONENT_TS: &str = r#"import type { Props } from "./NAME.proptypes";

import * as Styles from "./NAME.styles";

export default function NAME({ children }: Props) {
  return (
    <Styles.NAME>
      {children}
    </Styles.NAME>
  )
}
"#;