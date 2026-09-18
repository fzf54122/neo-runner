// @Time    : 2026/9/18 12:00
// @Author  : fzf
// @FileName: vite.config.js
// @Software: Claude Code

import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  base: process.env.VITE_PAGES_BASE || '/',
  plugins: [react()],
})
