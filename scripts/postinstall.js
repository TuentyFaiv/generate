const axios = require("axios");
const fs = require("fs");
const path = require("path");
const os = require("os");

async function downloadBinary() {
  const platform = os.platform();
  const bucket = "https://gvtmpjzawvdhojsxlece.supabase.co/storage/v1/object/public/cli/latest/";

  const executables = {
    win32: `${bucket}tfverse.exe`,
    linux: `${bucket}tfverse-linux`,
    darwin: `${bucket}tfverse-macos`,
  };

  if (!executables[platform]) {
    throw new Error(`Unsupported platform: ${platform}`)
  }

  const url = executables[platform];

  const response = await axios.get(url, { responseType: "arraybuffer" });

  const exts = {
    win32: ".exe",
    linux: "",
    darwin: "",
  };
  const namespace = {
    win32: "win",
    linux: "linux",
    darwin: "macos",
  }
  const cli = `tfverse${exts[platform]}`;
  const dir = `../bins/${namespace[platform]}`;

  const outputDir = path.join(__dirname, dir);
  fs.mkdirSync(outputDir, { recursive: true });

  const outputPath = path.join(outputDir, cli);
  fs.writeFileSync(outputPath, response.data);
  fs.chmodSync(outputPath, 0o755); // Make the binary executable
}

downloadBinary()
  .then(() => console.log("Binary downloaded and installed."))
  .catch((error) => console.error("Error downloading binary:", error));