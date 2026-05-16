import { Marked, type Tokens } from 'marked'
import hljs from 'highlight.js'

/**
 * Creates a pre-configured Marked instance with:
 * - GFM tables, task lists, strikethrough
 * - highlight.js code blocks
 * - Lazy-loaded images (src kept as-is; post-processed by ContentView)
 */
export function useMarkdown() {
  const md = new Marked()

  md.use({
    gfm: true,
    breaks: false,
    renderer: {
      /* ── Code blocks with syntax highlighting ──────── */
      code({ text, lang }: { text: string; lang?: string | undefined }) {
        const language = lang && hljs.getLanguage(lang) ? lang : undefined
        const highlighted = language
          ? hljs.highlight(text, { language }).value
          : hljs.highlightAuto(text).value
        const langClass = language ? ` language-${language}` : ''
        return `<pre><code class="hljs${langClass}">${highlighted}</code></pre>`
      },

      /* ── Images: keep href intact, ContentView post-processes ── */
      image({ href, title, text }: { href: string; title: string | null; text: string }) {
        const safeAlt = escapeHtml(text || '')
        const titleAttr = title ? ` title="${escapeHtml(title)}"` : ''
        return `<img src="${escapeHtml(href)}" alt="${safeAlt}"${titleAttr} loading="lazy" />`
      },

      /* ── External links open in new tab ── */
      link(token: Tokens.Link) {
        const inner = this.parser.parseInline(token.tokens)
        const titleAttr = token.title ? ` title="${escapeHtml(token.title)}"` : ''
        const href = token.href
        if (href.startsWith('http://') || href.startsWith('https://')) {
          return `<a href="${escapeHtml(href)}"${titleAttr} target="_blank" rel="noopener noreferrer">${inner}</a>`
        }
        return `<a href="${escapeHtml(href)}"${titleAttr}>${inner}</a>`
      },
    },
  })

  function parse(content: string): string {
    return md.parse(content, { async: false }) as string
  }

  return { parse }
}

/* ── Helpers ──────────────────────────────────────── */

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}
