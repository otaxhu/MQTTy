let obj;

try {
  const res = await fetch(
    "https://api.github.com/repos/otaxhu/MQTTy/releases/latest",
  );
  obj = await res.json();
} catch (e) {}

/** @type {string | undefined} */
export const windowsZip = obj?.assets?.find((asset) =>
  asset?.name?.endsWith("-win32-portable-x86_64.zip"),
)?.browser_download_url;
