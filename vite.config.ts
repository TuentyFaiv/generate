import { resolve } from "path";
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
      // UI Home
      "@home/hooks": resolve("src/ui/home/hooks/index.ts"),
      "@home/atoms": resolve("src/ui/home/atoms/index.ts"),
      "@home/molecules": resolve("src/ui/home/molecules/index.ts"),
      "@home/organisms": resolve("src/ui/home/organisms/index.ts"),
      "@home/styles": resolve("src/ui/home/styles/index.ts"),
      "@home/page": resolve("src/ui/home/+page.tsx"),
      "@home/layout": resolve("src/ui/home/+layout.tsx"),
      // NEXT_ALIAS

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
