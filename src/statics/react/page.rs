pub static PROPTYPES: &str = r#"// Change for interface if is an object
export type NAME_PASCALState = null;
"#;

pub static PAGE_TS: &str = r#"import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useAppContext } from "@contexts";

import type { NAME_PASCALState } from "@typing/pages/NAMESPACE";

import { Page } from "@NAMESPACE/styles";

export default function NAME_PASCALPage() {
  const { t } = useTranslation("NAMESPACE");
  const { global, dispatch } = useAppContext();
  const [state, setState] = useState<NAME_PASCALState>(null);

  return (
    <Page.NAME_PASCAL>
      {t("seo-title")}
    </Page.NAME_PASCAL>
  );
}
"#;

pub static PAGE: &str = r#"import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useAppContext } from "@contexts";

import { Page } from "@NAMESPACE/styles";

export default function NAME_PASCALPage() {
  const { t } = useTranslation("NAMESPACE");
  const { global, dispatch } = useAppContext();
  const [state, setState] = useState(null);

  return (
    <Page.NAME_PASCAL>
      {t("seo-title")}
    </Page.NAME_PASCAL>
  );
}
"#;

pub static STYLES: &str = r#"import styled from "@emotion/styled";

import * as responsive from "./NAME_PASCAL.styles.responsive";

export const NAME_PASCAL = styled.div`
  ${responsive.NAME_LOWER}
`;

export const Title = styled.div`
  ${responsive.title}
`;
"#;

pub static STYLES_RESPONSIVE: &str = r#"import { css } from "@emotion/react";
import { forsize } from "@mixins";

export const NAME_LOWER = forsize({
  "desktop-mid": css``,
  desktop: css``,
});

export const title = forsize({
  "desktop-mid": css``,
  desktop: css``,
});
"#;

pub static ROUTER: &str = r#"import { lazy, Suspense } from "react";
import { createBrowserRouter, Navigate } from "react-router-dom";

import { LoaderPortal } from "@sharing/atoms";

import Layout from "@sharing/layout";

/* NEXT_IMPORT */

export const router = createBrowserRouter([
  {
    path: "/",
    element: <Layout />,
    children: [
      /* NEXT_ROUTE */
    ]
  },
  {
    path: "*",
    element: <Navigate to="/" replace />
  },
]);
"#;

pub static ROUTE: &str = r#"{
        path: "NAME_LOWER",
        element: (
          <Suspense fallback={<LoaderPortal />}>
            <NAME_PASCAL />
          </Suspense>
        ),
      },
      /* NEXT_ROUTE */"#;

pub static LOCALE: &str = r#"{
  "seo-title": "NAME_PASCAL"
}
"#;

pub static I18N: &str = r#"import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";
import Backend from "i18next-http-backend";
import { enUS, es } from "date-fns/locale";
import config from "@config";

i18n
  .use(Backend)
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    debug: config.i18n_debug === "true",
    fallbackLng: config.i18n_fallback_lang,
    supportedLngs: config.i18n_langs,
    ns: [
      /* NEXT_LOCALE */
    ],
    nsSeparator: false,
    load: "currentOnly",
    interpolation: {
      escapeValue: false,
    },
    backend: {
      loadPath: "/locales/{{lng}}/{{ns}}.json",
    },
  });

const locales = {
  "en-US": enUS,
  es,
};

export { i18n, locales };
"#;

pub static TS_CONFIG: &str = r#"{
  "compilerOptions": {
    "target": "ESNext",
    "useDefineForClassFields": true,
    "lib": [
      "DOM",
      "DOM.Iterable",
      "ESNext"
    ],
    "allowJs": false,
    "skipLibCheck": true,
    "esModuleInterop": false,
    "allowSyntheticDefaultImports": true,
    "strict": true,
    "forceConsistentCasingInFileNames": true,
    "module": "ESNext",
    "moduleResolution": "Node",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "baseUrl": "./",
    "paths": {
      // Assets
      "@images/*": [
        "src/assets/images/*"
      ],
      "@icons/*": [
        "src/assets/images/icons/*"
      ],
      "@videos/*": [
        "src/assets/videos/*"
      ],
      "@fonts/*": [
        "src/assets/fonts/*"
      ],
      // Logic
      "@config": [
        "src/logic/config.ts"
      ],
      "@utils/*": [
        "src/logic/utils/*"
      ],
      "@typing/*": [
        "src/logic/typing/*"
      ],
      "@schemas/*": [
        "src/logic/schemas/*"
      ],
      "@services/*": [
        "src/logic/services/*"
      ],
      "@routes": [
        "src/logic/routes/router.tsx"
      ],
      "@contexts": [
        "src/logic/contexts/index.ts"
      ],
      "@hooks": [
        "src/logic/hooks/index.ts"
      ],
      "@hocs": [
        "src/logic/hocs/index.ts"
      ],
      /* NEXT_ALIAS */
      // UI Sharing
      "@sharing/atoms": [
        "src/ui/sharing/atoms/index.ts"
      ],
      "@sharing/molecules": [
        "src/ui/sharing/molecules/index.ts"
      ],
      "@sharing/organisms": [
        "src/ui/sharing/organisms/index.ts"
      ],
      "@sharing/layout": [
        "src/ui/sharing/+layout.tsx"
      ],
      "@styles": [
        "src/ui/sharing/styles/globals.ts"
      ],
      "@mixins": [
        "src/ui/sharing/styles/mixins.ts"
      ],
    }
  },
  "include": [
    "src",
    "vite.config.ts"
  ],
  "references": [
    {
      "path": "./tsconfig.node.json"
    }
  ]
}
"#;

pub static TS_ALIAS: &str = r#"// UI NAME_PASCAL
      "@NAMESPACE/hooks": [
        "src/ui/NAMESPACE/hooks/index.ts"
      ],
      "@NAMESPACE/atoms": [
        "src/ui/NAMESPACE/atoms/index.ts"
      ],
      "@NAMESPACE/molecules": [
        "src/ui/NAMESPACE/molecules/index.ts"
      ],
      "@NAMESPACE/organisms": [
        "src/ui/NAMESPACE/organisms/index.ts"
      ],
      "@NAMESPACE/styles": [
        "src/ui/NAMESPACE/styles/index.ts"
      ],
      "@NAMESPACE/page": [
        "src/ui/NAMESPACE/+page.tsx"
      ],
      "@NAMESPACE/layout": [
        "src/ui/NAMESPACE/+layout.tsx"
      ],
      /* NEXT_ALIAS */"#;

pub static VITE_CONFIG: &str = r#"import { resolve } from "path";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import eslint from "vite-plugin-eslint";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react(), eslint()],
  server: {
    port: 3000,
  },
  envPrefix: "PUB_",
  resolve: {
    alias: {
      // Assets
      "@images": resolve("src/assets/images/"),
      "@icons": resolve("src/assets/images/icons/"),
      "@videos": resolve("src/assets/videos/"),
      "@fonts": resolve("src/assets/fonts/"),
      // Logic
      "@config": resolve("src/logic/config.ts"),
      "@routes": resolve("src/logic/routes/router.tsx"),
      "@utils": resolve("src/logic/utils/"),
      "@typing": resolve("src/logic/typing/"),
      "@schemas": resolve("src/logic/schemas/"),
      "@services": resolve("src/logic/services/"),
      "@contexts": resolve("src/logic/contexts/index.ts"),
      "@hooks": resolve("src/logic/hooks/index.ts"),
      "@hocs": resolve("src/logic/hocs/index.ts"),
      /* NEXT_ALIAS */
      // UI Sharing
      "@sharing/atoms": resolve("src/ui/sharing/atoms/index.ts"),
      "@sharing/molecules": resolve("src/ui/sharing/molecules/index.ts"),
      "@sharing/organisms": resolve("src/ui/sharing/organisms/index.ts"),
      "@sharing/layout": resolve("src/ui/sharing/+layout.tsx"),
      "@styles": resolve("src/ui/sharing/styles/globals.ts"),
      "@mixins": resolve("src/ui/sharing/styles/mixins.ts"),
    },
  },
});
"#;

pub static VITE_ALIAS: &str = r#"// UI NAME_PASCAL
      "@NAMESPACE/hooks": resolve("src/ui/NAMESPACE/hooks/index.ts"),
      "@NAMESPACE/atoms": resolve("src/ui/NAMESPACE/atoms/index.ts"),
      "@NAMESPACE/molecules": resolve("src/ui/NAMESPACE/molecules/index.ts"),
      "@NAMESPACE/organisms": resolve("src/ui/NAMESPACE/organisms/index.ts"),
      "@NAMESPACE/styles": resolve("src/ui/NAMESPACE/styles/index.ts"),
      "@NAMESPACE/page": resolve("src/ui/NAMESPACE/+page.tsx"),
      "@NAMESPACE/layout": resolve("src/ui/NAMESPACE/+layout.tsx"),
      /* NEXT_ALIAS */"#;