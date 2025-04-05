export default function downloadBlob(
  blob: Blob,
  fileName: string = "Mastermind"
) {
  // This is really awkward to programatically download files, but its the only way to do it cleanly
  const objectUrl = URL.createObjectURL(blob);
  const a: HTMLAnchorElement = document.createElement("a") as HTMLAnchorElement;

  a.href = objectUrl;
  a.download = fileName;
  a.style = "display=none"; // Just to make extra sure it doesn't impact the layout
  document.body.appendChild(a);
  a.click();

  document.body.removeChild(a);
  URL.revokeObjectURL(objectUrl);
}
