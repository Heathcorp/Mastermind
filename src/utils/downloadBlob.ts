export default function downloadBlob(
  blob: Blob,
  fileName: string = "Mastermind"
) {
  // This is really awkward to programatically download files, but its the only way to do it cleanly
  const objectUrl = URL.createObjectURL(blob);
  const a: HTMLAnchorElement = document.createElement("a") as HTMLAnchorElement;

  a.href = objectUrl;
  a.download = fileName;
  document.body.appendChild(a);
  a.click();

  document.body.removeChild(a);
  URL.revokeObjectURL(objectUrl);
}
