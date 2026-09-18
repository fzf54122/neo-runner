// @Time    : 2026/9/18 12:00
// @Author  : fzf
// @FileName: App.jsx
// @Software: Claude Code

import { useEffect, useState } from 'react'
import { marked } from 'marked'
import hljs from 'highlight.js/lib/core'
import bash from 'highlight.js/lib/languages/bash'
import yaml from 'highlight.js/lib/languages/yaml'
import json from 'highlight.js/lib/languages/json'
import 'highlight.js/styles/github-dark.css'
import './App.css'
import {
  CARGO_CMD,
  INIT_CMD,
  INSTALL_CMD,
  RUN_CMD,
  REPO_URL,
  copy,
  docNav,
  docs,
  jsonSample,
  terminalLines,
  yamlSample,
} from './copy.js'

hljs.registerLanguage('bash', bash)
hljs.registerLanguage('yaml', yaml)
hljs.registerLanguage('json', json)

function getSystemTheme() {
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

function applyTheme(newTheme) {
  const resolvedTheme = newTheme === 'auto' ? getSystemTheme() : newTheme
  document.documentElement.setAttribute('data-theme', resolvedTheme)
}

function parseHash() {
  const raw = window.location.hash.replace(/^#\/?/, '')
  if (raw.startsWith('docs')) {
    const parts = raw.split('/')
    return { page: 'docs', doc: parts[1] || 'start' }
  }
  return { page: 'home', doc: 'start' }
}

async function copyText(value) {
  try {
    await navigator.clipboard.writeText(value)
    return true
  } catch {
    return false
  }
}

function prefersReducedMotion() {
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches
}

function TerminalDemo() {
  const [visibleCount, setVisibleCount] = useState(() =>
    prefersReducedMotion() ? terminalLines.length : 1,
  )

  useEffect(() => {
    if (prefersReducedMotion() || visibleCount >= terminalLines.length) {
      return undefined
    }

    const timer = window.setTimeout(() => {
      setVisibleCount((count) => count + 1)
    }, visibleCount === 1 ? 420 : 280)

    return () => window.clearTimeout(timer)
  }, [visibleCount])

  return (
    <div className="terminal-wrap">
      <div className="terminal" aria-label="neo-runner demo">
        <div className="terminal-bar">
          <span className="dot dot-red" />
          <span className="dot dot-yellow" />
          <span className="dot dot-green" />
          <span className="terminal-title">neo-runner</span>
        </div>
        <div className="terminal-body">
          {terminalLines.slice(0, visibleCount).map((line, index) => (
            <div
              key={line.text}
              className={`term-line term-${line.kind}`}
              style={{ animationDelay: `${index * 40}ms` }}
            >
              {line.text}
              {index === visibleCount - 1 && visibleCount < terminalLines.length ? (
                <span className="cursor" />
              ) : null}
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}

function readStoredTheme() {
  return localStorage.getItem('theme') || 'auto'
}

function readStoredLang() {
  return localStorage.getItem('lang') || 'zh'
}

function App() {
  const initial = parseHash()
  const [theme, setTheme] = useState(readStoredTheme)
  const [lang, setLang] = useState(readStoredLang)
  const [page, setPage] = useState(initial.page)
  const [activeDoc, setActiveDoc] = useState(initial.doc)
  const [searchQuery, setSearchQuery] = useState('')
  const [showThemeMenu, setShowThemeMenu] = useState(false)
  const [showLangMenu, setShowLangMenu] = useState(false)
  const [copiedKey, setCopiedKey] = useState('')
  const [stars, setStars] = useState(null)

  useEffect(() => {
    applyTheme(theme)
  }, [theme])

  useEffect(() => {
    const syncPageFromHash = () => {
      const next = parseHash()
      setPage(next.page)
      setActiveDoc(next.doc)
    }

    window.addEventListener('hashchange', syncPageFromHash)

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    const handleSystemThemeChange = () => {
      if ((localStorage.getItem('theme') || 'auto') === 'auto') {
        applyTheme('auto')
      }
    }

    mediaQuery.addEventListener('change', handleSystemThemeChange)
    return () => {
      window.removeEventListener('hashchange', syncPageFromHash)
      mediaQuery.removeEventListener('change', handleSystemThemeChange)
    }
  }, [])

  useEffect(() => {
    const titles = {
      zh: {
        home: 'neo-runner — 从终端验收',
        docs: '文档 - neo-runner',
      },
      en: {
        home: 'neo-runner — Done from the terminal',
        docs: 'Docs - neo-runner',
      },
    }
    document.title = titles[lang][page]
    document.documentElement.lang = lang === 'zh' ? 'zh-CN' : 'en'
  }, [page, lang])

  useEffect(() => {
    if (page !== 'docs') {
      return undefined
    }
    document.querySelectorAll('pre code').forEach((block) => {
      hljs.highlightElement(block)
    })
    return undefined
  }, [page, activeDoc, lang])

  useEffect(() => {
    let cancelled = false
    fetch('https://api.github.com/repos/fzf54122/neo-runner')
      .then((response) => (response.ok ? response.json() : null))
      .then((data) => {
        if (!cancelled && data && typeof data.stargazers_count === 'number') {
          setStars(data.stargazers_count)
        }
      })
      .catch(() => {})
    return () => {
      cancelled = true
    }
  }, [])

  const navigateTo = (nextPage, nextDoc = 'start') => {
    setPage(nextPage)
    setActiveDoc(nextDoc)
    window.location.hash = nextPage === 'home' ? '' : `/docs/${nextDoc}`
    window.scrollTo({ top: 0, behavior: 'smooth' })
  }

  const handleThemeChange = (newTheme) => {
    setTheme(newTheme)
    localStorage.setItem('theme', newTheme)
    applyTheme(newTheme)
    setShowThemeMenu(false)
  }

  const handleLangChange = (newLang) => {
    setLang(newLang)
    localStorage.setItem('lang', newLang)
    setShowLangMenu(false)
  }

  const handleCopy = async (key, value) => {
    const ok = await copyText(value)
    if (!ok) {
      return
    }
    setCopiedKey(key)
    window.setTimeout(() => setCopiedKey(''), 1400)
  }

  const t = copy[lang]
  const currentDoc = docs[lang][activeDoc] || docs[lang].start
  const filteredDocNav = docNav[lang].filter((item) =>
    item.title.toLowerCase().includes(searchQuery.toLowerCase()),
  )
  const commandRows = [
    { key: 'curl', label: 'curl', value: INSTALL_CMD },
    { key: 'cargo', label: 'cargo', value: CARGO_CMD },
    { key: 'init', label: 'init', value: INIT_CMD },
  ]

  return (
    <div className="app">
      <header className="header">
        <button type="button" className="header-left" onClick={() => navigateTo('home')}>
          <span className="logo-mark">&gt;_</span>
          <span className="logo">{t.logo}</span>
        </button>
        <nav className="nav">
          <button type="button" className="nav-link" onClick={() => navigateTo('docs')}>
            {t.docs}
          </button>
          <a className="nav-link" href={REPO_URL} target="_blank" rel="noopener noreferrer">
            {t.github}
          </a>
          <a className="star-chip" href={`${REPO_URL}/releases/tag/${t.versionLabel}`} target="_blank" rel="noopener noreferrer">
            {t.versionLabel}
          </a>
          <a className="star-chip" href={`${REPO_URL}/stargazers`} target="_blank" rel="noopener noreferrer">
            {t.starLabel}
            {stars !== null ? ` ${stars}` : ''}
          </a>
          <button type="button" className="btn-primary" onClick={() => navigateTo('docs', 'start')}>
            {t.getStarted}
          </button>

          <div className="dropdown">
            <button
              type="button"
              className="icon-btn"
              onClick={() => setShowThemeMenu(!showThemeMenu)}
              aria-label={t.theme.auto}
            >
              {theme === 'dark' ? '🌙' : theme === 'light' ? '☀️' : '💻'}
            </button>
            {showThemeMenu ? (
              <div className="dropdown-menu">
                <button type="button" onClick={() => handleThemeChange('light')}>
                  ☀️ {t.theme.light}
                </button>
                <button type="button" onClick={() => handleThemeChange('dark')}>
                  🌙 {t.theme.dark}
                </button>
                <button type="button" onClick={() => handleThemeChange('auto')}>
                  💻 {t.theme.auto}
                </button>
              </div>
            ) : null}
          </div>

          <div className="dropdown">
            <button
              type="button"
              className="icon-btn"
              onClick={() => setShowLangMenu(!showLangMenu)}
              aria-label="Language"
            >
              {lang === 'zh' ? '中' : 'EN'}
            </button>
            {showLangMenu ? (
              <div className="dropdown-menu">
                <button type="button" onClick={() => handleLangChange('zh')}>
                  简体中文
                </button>
                <button type="button" onClick={() => handleLangChange('en')}>
                  English
                </button>
              </div>
            ) : null}
          </div>
        </nav>
      </header>

      <main>
        {page === 'home' ? (
          <>
            <section className="hero">
              <p className="hero-kicker">{t.heroKicker}</p>
              <h1 className="hero-title">{t.heroTitle}</h1>
              <p className="hero-sub">{t.heroSub}</p>
              <div className="hero-command">
                <code>{INSTALL_CMD}</code>
                <button type="button" className="copy-btn" onClick={() => handleCopy('hero', INSTALL_CMD)}>
                  {copiedKey === 'hero' ? t.copiedLabel : t.copyLabel}
                </button>
              </div>
              <div className="hero-actions">
                <button type="button" className="btn-primary" onClick={() => navigateTo('docs', 'start')}>
                  {t.getStarted}
                </button>
                <a className="btn-ghost" href={REPO_URL} target="_blank" rel="noopener noreferrer">
                  {t.github} ↗
                </a>
              </div>
              <p className="hero-hint">{t.installHint}</p>
            </section>

            <TerminalDemo />

            <section className="section">
              <h2 className="section-title">{t.builtTitle}</h2>
              <div className="traits">
                {t.traits.map((item) => (
                  <article key={item.title} className="trait">
                    <h3>{item.title}</h3>
                    <p>{item.desc}</p>
                  </article>
                ))}
              </div>
            </section>

            <section className="section">
              <h2 className="section-title">{t.gateTitle}</h2>
              <p className="section-lead">{t.gateLead}</p>
              <div className="sample-grid">
                <article className="code-card">
                  <div className="code-card-bar">
                    <span>{t.yamlLabel}</span>
                    <code>.agents/loop.yaml</code>
                  </div>
                  <pre><code>{yamlSample}</code></pre>
                </article>
                <article className="code-card">
                  <div className="code-card-bar">
                    <span>{t.jsonLabel}</span>
                    <code>--output json</code>
                  </div>
                  <pre><code>{jsonSample}</code></pre>
                </article>
              </div>
              <ul className="field-list">
                {t.fields.map((field) => (
                  <li key={field.name}>
                    <code>{field.name}</code>
                    <span>{field.desc}</span>
                  </li>
                ))}
              </ul>
            </section>

            <section className="section">
              <h2 className="section-title">{t.stackTitle}</h2>
              <p className="section-lead">{t.stackLead}</p>
              <div className="traits">
                {t.stack.map((item) => (
                  <article key={item.title} className="trait">
                    <h3>{item.title}</h3>
                    <p>{item.desc}</p>
                  </article>
                ))}
              </div>
            </section>

            <section className="section">
              <h2 className="section-title">{t.runTitle}</h2>
              <p className="section-lead">{t.runLead}</p>
              <div className="hero-command command-inline">
                <code>{RUN_CMD}</code>
                <button type="button" className="copy-btn" onClick={() => handleCopy('run', RUN_CMD)}>
                  {copiedKey === 'run' ? t.copiedLabel : t.copyLabel}
                </button>
              </div>
              <div className="exits">
                {t.exits.map((item) => (
                  <article key={item.code} className="exit-card">
                    <code>{item.code}</code>
                    <h3>{item.title}</h3>
                    <p>{item.desc}</p>
                  </article>
                ))}
              </div>
              <button type="button" className="text-link" onClick={() => navigateTo('docs', 'contract')}>
                {t.moreDocs}
              </button>
            </section>

            <section className="section" id="install">
              <h2 className="section-title">{t.oneCommandTitle}</h2>
              <p className="section-lead">{t.oneCommandDesc}</p>
              <div className="command-block">
                {commandRows.map((row) => (
                  <div key={row.key} className="command-row">
                    <span>{row.label}</span>
                    <code>{row.value}</code>
                    <button type="button" className="copy-btn" onClick={() => handleCopy(row.key, row.value)}>
                      {copiedKey === row.key ? t.copiedLabel : t.copyLabel}
                    </button>
                  </div>
                ))}
              </div>
            </section>

            <section className="section">
              <h2 className="section-title">{t.installTitle}</h2>
              <div className="installs">
                {t.installs.map((item) => (
                  <article key={item.title} className="install-card">
                    <h3>{item.title}</h3>
                    <p>{item.desc}</p>
                  </article>
                ))}
              </div>
            </section>
          </>
        ) : null}

        {page === 'docs' ? (
          <section className="docs-layout">
            <aside className="docs-sidebar">
              <div className="docs-search">
                <input
                  type="text"
                  placeholder={t.docsSearch}
                  value={searchQuery}
                  onChange={(event) => setSearchQuery(event.target.value)}
                />
              </div>
              <nav className="docs-nav">
                {filteredDocNav.map((item) => (
                  <button
                    key={item.id}
                    type="button"
                    className={`docs-nav-item ${activeDoc === item.id ? 'active' : ''}`}
                    onClick={() => navigateTo('docs', item.id)}
                  >
                    {item.title}
                  </button>
                ))}
              </nav>
            </aside>
            <article className="docs-content">
              <h1>{currentDoc.title}</h1>
              <div
                className="markdown-body"
                dangerouslySetInnerHTML={{ __html: marked(currentDoc.content) }}
              />
            </article>
          </section>
        ) : null}
      </main>

      <footer className="footer">
        <p>{t.footer}</p>
        <div className="footer-links">
          <a href={REPO_URL} target="_blank" rel="noopener noreferrer">
            GitHub
          </a>
          <button type="button" onClick={() => navigateTo('docs')}>
            {t.docs}
          </button>
          <a href={`${REPO_URL}/releases`} target="_blank" rel="noopener noreferrer">
            Releases
          </a>
        </div>
      </footer>
    </div>
  )
}

export default App
