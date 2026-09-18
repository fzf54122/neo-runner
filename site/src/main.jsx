// @Time    : 2026/9/18 12:00
// @Author  : fzf
// @FileName: main.jsx
// @Software: Claude Code

import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.jsx'

createRoot(document.getElementById('root')).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
