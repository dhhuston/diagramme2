/** Whether a datasheet URL can become an Excel hyperlink in exports. */
export function isDatasheetHyperlinkUrl(url: string): boolean {
  const trimmed = url.trim()
  return /^https?:\/\//i.test(trimmed)
}
